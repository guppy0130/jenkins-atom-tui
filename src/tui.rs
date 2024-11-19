use std::{error::Error, io, panic};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::Backend, Terminal};

use crate::{app::App, event::EventHandler, ui};

#[derive(Debug)]
pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn Error>> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            // TODO: figure out how to show the cursor while respecting lifetimes
            panic_hook(panic);
        }));
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<(), Box<dyn Error>> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    pub fn reset() -> Result<(), Box<dyn Error>> {
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Box<dyn Error>> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}
