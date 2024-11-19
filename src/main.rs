use std::sync::LazyLock;
use std::{error::Error, io};

use clap::Parser;
use event::{Event, EventHandler};
use expanduser::expanduser;
use handler::handle_key_events;
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::app::App;
use crate::tui::Tui;

pub mod app;
pub mod event;
pub mod handler;
pub mod jenkins;
pub mod tui;
pub mod ui;

static DEFAULT_JENKINS_CONFIG_PATH: LazyLock<String> = LazyLock::new(|| {
    expanduser("~/.config/jenkins_jobs/jenkins_jobs.ini")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
});

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = DEFAULT_JENKINS_CONFIG_PATH.to_string())]
    jenkins_config_path: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut app = App::new(args.jenkins_config_path);

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => {
                // keypress may cause additional events for processing
                if let Ok(Some(event)) = handle_key_events(key_event, &mut app).await {
                    tui.events.push_event(event);
                }
            }
            Event::RefreshJobsForServer => app.refresh_jobs().await?,
            Event::RefreshLogsForJob => app.refresh_logs().await?,
            // TODO: potentially handle other events
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
