use std::{
    collections::BTreeMap, error::Error, fmt::Display, fs, path::Path, str::FromStr, sync::LazyLock,
};

use atom_syndication::{Entry, Feed};
use chrono::{DateTime, FixedOffset};
use ratatui::{style::Color, widgets::ListItem};
use regex::Regex;
use reqwest::{Client, Response, Url};
use serde::Deserialize;

/// describes the state of a build
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuildState {
    Success,
    Failure,
    Unknown,
}

impl FromStr for BuildState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return match s.to_lowercase().as_str() {
            "stable" => Ok(BuildState::Success),
            "back to normal" => Ok(BuildState::Success),
            "broken" => Ok(BuildState::Failure), // TODO: is this correct?
            _ => Ok(BuildState::Unknown),
        };
    }
}

/// describes a build
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JenkinsResult {
    /// job name
    pub name: String,
    /// job build number
    pub build_number: u128,
    /// did it succeed
    pub build_state: BuildState,
    /// when was job info updated (may change if job still running)
    pub updated: DateTime<FixedOffset>,
    /// URL to job
    pub link: Url,
    /// job logs (might have a lot)
    pub logs: String,
}

/// How to parse the rss entry's title to get build name, number, and status
static BUILD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?P<name>.*) #(?P<build_number>\d+) \((?P<build_state>stable|broken|back to normal).*",
    )
    .unwrap()
});

impl Default for JenkinsResult {
    fn default() -> Self {
        JenkinsResult {
            name: String::new(),
            build_number: 0,
            build_state: BuildState::Failure,
            updated: DateTime::UNIX_EPOCH.fixed_offset(),
            link: Url::from_file_path("/dev/null").unwrap(),
            logs: String::new(),
        }
    }
}

impl TryFrom<Entry> for JenkinsResult {
    type Error = Box<dyn Error>;
    fn try_from(value: Entry) -> Result<Self, Self::Error> {
        let captures = BUILD_REGEX
            .captures(&value.title)
            .ok_or("Entry title doesn't match regex to extract build info")?;
        Ok(JenkinsResult {
            name: captures
                .name("name")
                .ok_or("Missing build name")?
                .as_str()
                .to_string(),
            build_number: captures
                .name("build_number")
                .ok_or("Missing build number")?
                .as_str()
                .parse()?,
            build_state: captures
                .name("build_state")
                .ok_or("Missing build state")?
                .as_str()
                .parse()?,
            updated: value.updated,
            link: Url::parse(&value.links[0].href)?,

            // TODO: figure out if hydrating the logs now would be too expensive
            ..Default::default()
        })
    }
}

impl<'a> From<JenkinsResult> for ListItem<'a> {
    fn from(value: JenkinsResult) -> Self {
        Self::new(format!("{} #{}", value.name, value.build_number)).style(
            match value.build_state {
                BuildState::Success => Color::Green,
                BuildState::Failure => Color::Red,
                _ => Color::Yellow,
            },
        )
    }
}

impl Display for JenkinsResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} #{}", self.name, self.build_number))
    }
}

impl PartialOrd for JenkinsResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// TODO: maybe sort by build number?
impl Ord for JenkinsResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.updated.cmp(&other.updated)
    }
}

impl JenkinsResult {
    /// hydrate logs on individual results one at a time, because getting them all at once might be
    /// really expensive.
    pub async fn hydrate_logs(&mut self, server: &mut JenkinsServer) -> Result<(), Box<dyn Error>> {
        let logs_url = self.link.join("consoleText").unwrap();
        self.logs = server
            .request_with_auth(logs_url.as_str())
            .await?
            .text()
            .await?;
        Ok(())
    }
}

// TODO: does it make sense to store the jobs in here too?
/// how to connect to jenkins
#[derive(Debug, Deserialize)]
pub struct JenkinsServer {
    url: String,
    user: String,
    password: String,

    #[serde(skip, default)]
    client: Option<Client>,
}

impl Display for JenkinsServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}:{}@{}",
            self.user,
            "x".repeat(self.password.len()),
            self.url
        ))
    }
}

impl TryFrom<BTreeMap<String, String>> for JenkinsServer {
    type Error = String;

    fn try_from(value: BTreeMap<String, String>) -> Result<Self, Self::Error> {
        let url = value.get("url").ok_or("Missing URL")?.to_owned();
        let user = value.get("user").ok_or("Missing user")?.to_owned();
        let password = value.get("password").ok_or("Missing password")?.to_owned();
        Ok(Self {
            url,
            user,
            password,
            client: None,
        })
    }
}

impl JenkinsServer {
    /// make a request to the Jenkins server with basic auth
    pub async fn request_with_auth(
        &mut self,
        relative_url: &str,
    ) -> Result<Response, Box<dyn Error>> {
        // panic!("{:?}", self.client);
        let mut url = Url::parse(&self.url)?;
        url = url.join(relative_url)?;
        let client = self.client.get_or_insert(reqwest::Client::new());
        Ok(client
            .get(url)
            .basic_auth(self.user.clone(), Some(self.password.clone()))
            .send()
            .await?)
    }
}

/// parse `rssAll.atom` to get job history/results
pub async fn fetch_jenkins_results(
    jenkins_server: &mut JenkinsServer,
) -> Result<impl Iterator<Item = JenkinsResult>, Box<dyn Error>> {
    let xml_resp = jenkins_server
        .request_with_auth("/rssAll")
        .await?
        .text()
        .await?;
    let feed = Feed::from_str(&xml_resp).unwrap();

    Ok(feed
        .entries
        .into_iter()
        .filter_map(|entry| JenkinsResult::try_from(entry).ok()))
}

/// read the specified JJB config file and turn it into JenkinsServers
pub fn read_jenkins_config_file<P: AsRef<Path>>(
    path: P,
) -> Result<BTreeMap<String, JenkinsServer>, Box<dyn Error>> {
    let jenkins_file_content = fs::read_to_string(path)?;
    let config_with_extras: BTreeMap<String, BTreeMap<String, String>> =
        serde_ini::from_str(&jenkins_file_content)?;
    let parsed_jenkins_config: BTreeMap<String, JenkinsServer> = config_with_extras
        .into_iter()
        .filter_map(|(k, v)| {
            if let Ok(server) = JenkinsServer::try_from(v) {
                return Some((k, server));
            }
            None
        })
        .collect();
    Ok(parsed_jenkins_config)
}
