use async_std::path::PathBuf;
use structopt::StructOpt;

/// A world server for Wrath of the Lich King.
/// Run with no arguments to start the server.
#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(default_value = "config.yaml")]
    pub config: PathBuf,

    #[structopt(subcommand)]
    pub command: Option<OptCommand>,
}

#[derive(StructOpt, Debug)]
pub enum OptCommand {
    /// Generate a new config file.
    Init,
}
