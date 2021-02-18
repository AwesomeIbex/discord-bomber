use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
name = "Discord Bomber",
about = "A discord automation bot",
version = "v0.0.1",
author = "Donovan Dall - awesomealpineibex@gmail.com"
)]
pub struct Opts {
    /// Number of accounts to create
    #[structopt(short = "a", long = "accounts")]
    pub accounts: i32,

    /// Server invite link
    #[structopt(short = "i", long = "invite_link")]
    pub invite_link: String, //TODO use me
}

pub fn get_opts_args() -> Opts {
    let opts = Opts::from_args();
    opts
}
