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
    /// Execute a command directly.
    Exec(Command),
    /// Run the server.
    Run,
    /// Generate a new config file.
    Init,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Account {
        #[structopt(subcommand)]
        command: AccountCommand,
    },
    /// Exit the server
    #[structopt(aliases=&["exit"])]
    Shutdown,
}

/// Commands for managing accounts
#[derive(StructOpt, Debug)]
pub enum AccountCommand {
    /// Create a new account
    Create {
        /// The username of the new account
        username: String,
        /// The password to use
        password: String,
        /// The email address
        email: String,
    },
}
