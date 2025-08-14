use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(multicall = true)]
pub struct MainCli {
    #[command(subcommand)]
    pub command: MainCommands,
}

#[derive(Debug, Subcommand)]
pub enum MainCommands {
    /// Initializes the sewer security system
    Init,
    /// Pauses the game
    Pause,
}
