#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

pub mod commands;
pub mod crates;

use std::process::exit;
use structopt::StructOpt;

pub type DargoResult<T> = anyhow::Result<T>;
use crate::commands::Command;

fn main() {
    init_logger();

    let command_result = match Command::from_args() {
        Command::Upgrade(x) => x.run(),
        Command::Add(x) => x.run(),
        Command::Rm(x) => x.run(),
    };
    if let Err(error) = command_result {
        error!("{}", error);
        exit(1);
    }
}

fn init_logger() {
    flexi_logger::Logger::try_with_env_or_str("warn")
        .unwrap()
        .format(|w, _, record| write!(w, "[{}] {}", record.level(), &record.args()))
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
}
