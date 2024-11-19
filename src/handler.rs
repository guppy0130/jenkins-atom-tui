use std::error::Error;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{app::App, event::Event};

/// handle keydown events. since you may end up needing to do things as a result of a keypress, you
/// may opt to return an Event that'll be added to the queue for later processing.
pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
) -> Result<Option<Event>, Box<dyn Error>> {
    match key_event.code {
        // exit the app with esc, q, or <C-c>
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') => {
            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                app.quit();
            }
        }

        // handle going to different panes
        KeyCode::Char('1') => app.set_active_pane(1),
        KeyCode::Char('2') => app.set_active_pane(2),
        KeyCode::Char('3') => app.set_active_pane(3),

        // don't do anything else with the other keys?
        _ => {}
    }

    // TODO: figure out if there's a better way to achieve per-pane logic
    match app.active_pane {
        1 => match key_event.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.servers.server_state.select_next();
                return Ok(Some(Event::RefreshJobsForServer));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.servers.server_state.select_previous();
                return Ok(Some(Event::RefreshJobsForServer));
            }
            KeyCode::Char('r') => app.refresh_servers(),
            _ => {}
        },
        2 => {
            if let Some((stateful_job, _)) = app.get_current_server_jobs() {
                match key_event.code {
                    // TODO: figure out if we should _not_ refresh logs if there is _already_ logs,
                    // unless the user explicitly requests it
                    KeyCode::Char('j') | KeyCode::Up => {
                        stateful_job.job_state.select_next();
                        return Ok(Some(Event::RefreshLogsForJob));
                    }
                    KeyCode::Char('k') | KeyCode::Down => {
                        stateful_job.job_state.select_previous();
                        return Ok(Some(Event::RefreshLogsForJob));
                    }
                    // TODO: should this refresh logs too?
                    KeyCode::Char('r') => return Ok(Some(Event::RefreshJobsForServer)),
                    _ => {}
                }
            }
        }
        3 => {
            if let Some((stateful_job, _)) = app.get_current_server_jobs() {
                if stateful_job.job_state.selected().is_some() {
                    match key_event.code {
                        KeyCode::Char('j') | KeyCode::Down => app.log_scroll_state.scroll_down(),
                        KeyCode::Char('k') | KeyCode::Up => app.log_scroll_state.scroll_up(),
                        KeyCode::Char('h') | KeyCode::Left => app.log_scroll_state.scroll_left(),
                        KeyCode::Char('l') | KeyCode::Right => app.log_scroll_state.scroll_right(),
                        KeyCode::Char('w') => app.wrap_logs = !app.wrap_logs,
                        KeyCode::PageDown => app.log_scroll_state.scroll_page_down(),
                        KeyCode::PageUp => app.log_scroll_state.scroll_page_up(),
                        _ => {}
                    }
                }
            }
        }
        // nothing else to do I suppose
        _ => {}
    }

    Ok(None)
}
