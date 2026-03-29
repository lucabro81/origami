mod commands;

use clap::Parser;
use commands::Command;

#[derive(Parser, Debug)]
#[command(name = "origami", version, about = "The Origami framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub fn run() {
    let cli = Cli::parse();
    match cli.command {
        Command::Dev(args) => commands::dev::run(args),
        Command::Build(args) => commands::build::run(args),
        Command::Check(args) => commands::check::run(args),
        Command::Test(args) => commands::test::run(args),
        Command::Init(args) => commands::init::run(args),
        Command::UnsafeReport(args) => commands::unsafe_report::run(args),
    }
}

#[cfg(test)]
mod tests;
