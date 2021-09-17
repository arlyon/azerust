use async_std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(default_value = "config.yaml")]
    pub config: PathBuf,

    #[structopt(subcommand)]
    pub command: OptCommand,
}

#[derive(StructOpt, Debug)]
pub enum OptCommand {
    /// Run the server.
    Run,
    /// Generate a new config file.
    Init,
}
