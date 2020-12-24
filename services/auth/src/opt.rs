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
    /// Run the server with a tui.
    Tui,
    /// Run the server with a repl.
    Repl,
    /// Run the server just viewing logs.
    Log,
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
