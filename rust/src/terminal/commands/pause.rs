use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(multicall = true)]
pub struct PauseCli {
    #[command(subcommand)]
    pub command: PauseCommands,
}

#[derive(Debug, Subcommand)]
pub enum PauseCommands {
    /// Un-pauses the game
    UnPause,
}
