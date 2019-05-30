pub mod add;
pub mod upgrade;

use structopt::StructOpt;

/// Some useful third-party tools for Cargo
#[derive(Debug, StructOpt)]
#[structopt(after_help = "issue report: https://github.com/DCjanus/dargo")]
pub enum Command {
    /// Upgrade dependencies in your Cargo.toml
    #[structopt(name = "upgrade")]
    Upgrade(self::upgrade::Upgrade),

    /// Add dependencies to your Cargo.toml
    #[structopt(name = "add")]
    Add(self::add::Add),
}
