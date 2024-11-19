use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::{Block, List, Paragraph, Wrap},
    Frame,
};
use tui_scrollview::ScrollView;

use crate::{app::App, jenkins::JenkinsResult};

static ACCENT_COLOR: Color = Color::Magenta;
static HIGHLIGHT_SYMBOL: &str = ">> ";

pub fn render(app: &mut App, frame: &mut Frame) {
    let [main_app, status] =
        Layout::vertical([Constraint::Percentage(100), Constraint::Min(3)]).areas(frame.area());
    let [server_list, job_pane] =
        Layout::horizontal([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)]).areas(main_app);
    let [job_list, job_logs] =
        Layout::vertical([Constraint::Percentage(20), Constraint::Fill(1)]).areas(job_pane);

    let status_text =
        Paragraph::new(app.status.to_string()).block(Block::bordered().title("Status"));

    let mut server_list_block = Block::bordered().title("Server List [1]");
    let server_list_list = List::new(app.servers.servers.keys().cloned())
        .highlight_symbol(HIGHLIGHT_SYMBOL)
        .highlight_style(ACCENT_COLOR)
        .repeat_highlight_symbol(true);
    frame.render_stateful_widget(
        server_list_list,
        server_list_block.inner(server_list),
        &mut app.servers.server_state,
    );

    let mut job_list_block = Block::bordered().title("Job List [2]");
    if let Some((stateful_jobs, _)) = app.get_current_server_jobs() {
        // FIXME: weird borrow-checker nonsense
        let mut cache_vec = vec![JenkinsResult::default(); stateful_jobs.jobs.len()];
        cache_vec.clone_from_slice(&stateful_jobs.jobs);
        let job_list_list = List::new(cache_vec)
            .highlight_symbol(HIGHLIGHT_SYMBOL)
            // .highlight_style(ACCENT_COLOR)
            .repeat_highlight_symbol(true);
        frame.render_stateful_widget(
            job_list_list,
            job_list_block.inner(job_list),
            &mut stateful_jobs.job_state,
        );
    }

    let mut job_logs_block = Block::bordered().title("Job Logs [3]");
    let job_logs_inner = job_logs_block.inner(job_logs);
    // if there's no selected job, don't bother generating the paragraph for the
    // logs nor the scrollview.
    if let Some((stateful_job, _)) = app.get_current_server_jobs() {
        if let Some(job_idx) = stateful_job.job_state.selected() {
            if !stateful_job.jobs[job_idx].logs.is_empty() {
                // TODO: fight borrow checker on this clone
                let mut paragraph = Paragraph::new(stateful_job.jobs[job_idx].logs.clone());

                // allow hitting `w` to wrap the text.
                let mut width: u16 = paragraph.line_width().try_into().unwrap();
                // if we're configured to wrap the text, the max width is the container, and we have
                // to tell the paragraph that it's gonna be wrapped
                if app.wrap_logs {
                    paragraph = paragraph.wrap(Wrap { trim: false });
                    width = job_logs_inner.as_size().width;
                }

                let paragraph_rect =
                    Rect::new(0, 0, width, paragraph.line_count(width).try_into().unwrap());

                // the scroll view must be sized to the content!
                let mut job_logs_scrollview = ScrollView::new(paragraph_rect.as_size());
                // render the paragraph into the scroll view
                job_logs_scrollview.render_widget(paragraph, paragraph_rect);
                // render the scroll view into the frame. the scroll view should be larger than the
                // job_logs_inner, and the scroll view will add scrollbars as necessary.
                frame.render_stateful_widget(
                    job_logs_scrollview,
                    job_logs_inner,
                    &mut app.log_scroll_state,
                );
            }
        }
    }

    // highlight the active pane
    match app.active_pane {
        1 => server_list_block = server_list_block.border_style(ACCENT_COLOR),
        2 => job_list_block = job_list_block.border_style(ACCENT_COLOR),
        3 => job_logs_block = job_logs_block.border_style(ACCENT_COLOR),
        _ => {}
    }

    frame.render_widget(status_text, status);
    frame.render_widget(server_list_block, server_list);
    frame.render_widget(job_list_block, job_list);
    frame.render_widget(job_logs_block, job_logs);
}
