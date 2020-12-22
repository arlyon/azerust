use std::{net::IpAddr, path::PathBuf};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opt {
    config: PathBuf,
}
