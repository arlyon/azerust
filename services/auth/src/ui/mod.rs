use async_std::channel::Receiver;
use async_std::channel::Sender;

use anyhow::Result;
use async_trait::async_trait;

mod event;
mod repl;
mod tui;

pub use self::repl::Repl;
pub use self::tui::Tui;

use crate::authserver::{Command, Response};

/// A generic trait for UIs showing the state of a server. It may send commands,
/// and responds to Responses.
#[async_trait]
pub trait UI {
    async fn start(&self, s: &Sender<Command>, r: &Receiver<Response>) -> Result<()>;
}
