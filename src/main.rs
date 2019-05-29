#[macro_use]
extern crate log;

pub mod commands;
pub mod crates;

use std::process::exit;
use structopt::StructOpt;

pub type DargoResult<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, StructOpt)]
enum Args {
    #[structopt(name = "upgrade")]
    Upgrade(commands::upgrade::Upgrade),
}

fn main() {
    init_logger();

    let command_result = match Args::from_args() {
        Args::Upgrade(x) => x.run(),
    };
    if let Err(error) = command_result {
        error!("{}", error);
        exit(1);
    }
}

fn init_logger() {
    flexi_logger::Logger::with_env_or_str("warn")
        .format(|w, now, record| {
            write!(
                w,
                "[{}] {} [{}:{}] {}",
                now.now().format("%Y-%m-%d %H:%M:%S %:z"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                record.line().unwrap_or(0),
                &record.args()
            )
        })
        .start()
        .unwrap_or_else(|e| panic!("Logger initialization failed with {}", e));
}
