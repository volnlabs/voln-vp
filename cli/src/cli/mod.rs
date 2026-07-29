use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "voln-vp",
    version,
    about = "Virtual Platform for axiomOS Development"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Verify required simulators and toolchains are installed
    Doctor,
    /// Run a board under a backend
    Run,
    /// Run test suites for a board under a backend
    Test,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Command::Doctor => todo!("Task 2.7"),
        Command::Run => todo!("Task 2.5"),
        Command::Test => todo!("Task 2.5"),
    }
}
