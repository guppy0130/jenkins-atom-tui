use std::{error::Error, time::Duration};

use crossterm::event::{EventStream, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::{select, sync::mpsc};

#[derive(Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
    RefreshJobsForServer,
    RefreshLogsForJob,
}

#[allow(dead_code)] // TODO: figure out where this could be used if necessary
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::UnboundedSender<Event>,
    receiver: mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();

        let handler = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let event = reader.next().fuse();
                select! {
                    _ = _sender.closed() => { break; },
                    _ = tick_delay => { _sender.send(Event::Tick).unwrap() },
                    Some(Ok(evt)) = event => {
                        match evt {
                            crossterm::event::Event::FocusGained => {},
                            crossterm::event::Event::FocusLost => {},
                            crossterm::event::Event::Key(key_event) => {
                                if key_event.kind == KeyEventKind::Press {
                                    _sender.send(Event::Key(key_event)).unwrap()
                                }
                            },
                            crossterm::event::Event::Mouse(_) => {},
                            crossterm::event::Event::Paste(_) => {},
                            crossterm::event::Event::Resize(x, y) => {_sender.send(Event::Resize(x, y)).unwrap()},
                        }
                    }
                }
            }
        });

        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Result<Event, Box<dyn Error>> {
        self.receiver.recv().await.ok_or("IO Error?".into())
    }

    /// may need to add events for processing in future ticks
    pub fn push_event(&self, event: Event) {
        self.sender.send(event).unwrap();
    }
}
