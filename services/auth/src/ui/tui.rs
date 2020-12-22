use async_std::channel::{Receiver, Sender};
use async_trait::async_trait;
use io::{BufWriter, Write};
use std::{
    io,
    sync::{Arc, Mutex},
};

use tui_logger::TuiLoggerSmartWidget;

use anyhow::Result;
use colored::*;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

impl From<&Response> for ListItem<'_> {
    fn from(r: &Response) -> Self {
        ListItem::new(match r {
            Response::Ready => format!("Server ready."),
            Response::Update(u) => format!("{}", u),
            Response::Complete(c) => format!("{}", c.green()),
            Response::Error(e) => format!("{}", e.red()),
        })
    }
}

use super::{
    event::{Event, Events},
    UI,
};
use crate::authserver::{Command, Response};

struct Realm {
    name: String,
    pop: u32,
    max: u32,
}

pub struct Tui;

#[async_trait]
impl UI for Tui {
    async fn start(&self, s: &Sender<Command>, r: &Receiver<Response>) -> Result<()> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut responses: Vec<ListItem> = vec![];
        let realms = vec![
            Realm {
                name: "Stormhammer".to_string(),
                pop: 3245,
                max: 5000,
            },
            Realm {
                name: "Ironfist".to_string(),
                pop: 245,
                max: 300,
            },
        ];

        // Setup event handlers
        let events = Events::new();

        loop {
            loop {
                let x = r.try_recv();
                match x {
                    Ok(r) => responses.insert(0, (&r).into()),
                    Err(e) => match e {
                        async_std::channel::TryRecvError::Empty => break,
                        async_std::channel::TryRecvError::Closed => return Ok(()),
                    },
                }
            }

            terminal.draw(|f| {
                let realm_count = realms.len() as u16;

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(2 + realm_count),
                            Constraint::Min(0),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let block = Block::default().title("Realms").borders(Borders::ALL);
                let realms = List::new(
                    realms
                        .iter()
                        .map(|r| ListItem::new(format!("{}: {}/{}", r.name, r.pop, r.max)))
                        .collect::<Vec<_>>(),
                )
                .block(block);

                f.render_widget(realms, chunks[0]);

                let instructions = Text::from(Spans::from(vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, and "),
                    Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to write a command. To inspect a server,"),
                ]));

                f.render_widget(Paragraph::new(instructions), chunks[2]);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[1]);

                let block = Block::default().title("Logs").borders(Borders::ALL);

                f.render_widget(block, chunks[0]);

                let block = Block::default().title("Commands").borders(Borders::ALL);
                let logs = List::new(responses.clone()).block(block);
                f.render_widget(logs, chunks[1]);
            })?;

            // Handle input
            if let Event::Input(input) = events.next()? {
                if let Key::Char('q') = input {
                    async_std::task::block_on(async { s.send(Command::ShutDown).await });
                }
            }
        }

        Ok(())
    }
}

// create a new collector
// feed data into list window
