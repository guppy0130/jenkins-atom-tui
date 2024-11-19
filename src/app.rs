use std::{
    collections::{BTreeMap, HashMap},
    error::Error,
    path::{Path, PathBuf},
};

use ratatui::widgets::ListState;
use tui_scrollview::ScrollViewState;

use crate::jenkins::{
    fetch_jenkins_results, read_jenkins_config_file, JenkinsResult, JenkinsServer,
};

#[derive(Debug, Default)]
pub struct StatefulServers {
    /// map of server name (ini/toml table title) to connection info
    pub servers: BTreeMap<String, JenkinsServer>,
    /// tracks the active jenkins server
    pub server_state: ListState,
}

// TODO: figure out if we should shove this into the JenkinsServer struct
#[derive(Debug, Clone, Default)]
pub struct StatefulJobs {
    /// jobs in this server
    pub jobs: Vec<JenkinsResult>,
    /// tracks the selected job
    pub job_state: ListState,
}

/// application state
#[derive(Debug)]
pub struct App {
    /// if the app is running (set to quit to exit)
    pub running: bool,
    /// a status message to display
    pub status: String,
    /// tracks the active pane
    pub active_pane: u8,

    /// path to JJB config
    jenkins_config_path: PathBuf,
    pub servers: StatefulServers,
    pub jobs: HashMap<usize, StatefulJobs>,
    pub log_scroll_state: ScrollViewState,
    pub wrap_logs: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            status: "ESC, CTRL+C, or q to exit app".to_string(),
            active_pane: 1,
            jenkins_config_path: PathBuf::new(),
            servers: StatefulServers::default(),
            jobs: HashMap::new(),
            log_scroll_state: ScrollViewState::new(),
            wrap_logs: false,
        }
    }
}

impl App {
    pub fn new<P: AsRef<Path>>(jenkins_config_path: P) -> Self {
        let mut returnable = Self {
            jenkins_config_path: jenkins_config_path.as_ref().into(),
            ..Default::default()
        };
        // TODO: move to async?
        returnable.refresh_servers();
        returnable
    }

    /// set the status message (but if you call twice before tick, you may miss first call)
    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }

    /// sets the active pane
    pub fn set_active_pane(&mut self, active_pane: u8) {
        if !(1..=3).contains(&active_pane) {
            self.status = format!("{} is an invalid pane number", active_pane);
            return;
        }
        self.status = format!("Setting active pane to {}", active_pane);
        self.active_pane = active_pane;
    }

    /// read the JJB config file from disk to configure the servers available to the GUI
    pub fn refresh_servers(&mut self) {
        self.set_status("Reading JJB config file for servers");
        self.servers.servers = read_jenkins_config_file(&self.jenkins_config_path).unwrap();
        self.servers.server_state.select(None);
        self.set_status(&format!("Found {} servers", self.servers.servers.len()));
    }

    /// return jobs associated with the selected jenkins instance, if one is selected
    pub fn get_current_server_jobs(&mut self) -> Option<(&mut StatefulJobs, &mut JenkinsServer)> {
        if let Some(idx) = self.servers.server_state.selected() {
            return Some((
                self.jobs.get_mut(&idx)?,
                self.servers.servers.values_mut().nth(idx).unwrap(),
            ));
        }
        None
    }

    /// refresh jobs for just the instance that's selected
    pub async fn refresh_jobs(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(idx) = self.servers.server_state.selected() {
            let jenkins_server = self.servers.servers.values_mut().nth(idx).unwrap();
            let job_entry = self.jobs.entry(idx).or_default();
            job_entry.jobs = fetch_jenkins_results(jenkins_server).await?.collect();
            job_entry.job_state.select(None);
            let updated_job_counter = job_entry.jobs.len();
            self.set_status(&format!(
                "Fetched {} job results from {}",
                updated_job_counter,
                self.servers.servers.values().nth(idx).unwrap()
            ));
        }
        Ok(())
    }

    /// refresh job logs for just the job that's selected
    pub async fn refresh_logs(&mut self) -> Result<(), Box<dyn Error>> {
        let mut selected_job_name = String::new();
        if let Some((stateful_jobs, jenkins_server)) = self.get_current_server_jobs() {
            if let Some(job_idx) = stateful_jobs.job_state.selected() {
                let selected_job = stateful_jobs.jobs.get_mut(job_idx).unwrap();
                selected_job.hydrate_logs(jenkins_server).await?;
                selected_job_name = format!("{}", selected_job);
            }
        }
        self.set_status(&format!("Fetched logs for {}", selected_job_name));
        Ok(())
    }

    pub fn tick(&self) {}

    /// set running to false to quit the application.
    pub fn quit(&mut self) {
        self.set_status("Quitting");
        self.running = false;
    }
}
