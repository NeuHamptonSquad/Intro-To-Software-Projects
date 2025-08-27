use clap::{Args, Parser, Subcommand, ValueEnum};

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
    /// Opens/Closes a gate
    Gate(GateArgs),
    /// Zooms in on an area, allowing you to operate
    /// on said area
    Zoom {
        /// The area to zoom in on
        #[arg(value_enum)]
        area: Area,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Area {
    MainView,
    Room1,
    Room2,
    Room3,
    Room4,
}

#[derive(Debug, Args)]
pub struct GateArgs {
    /// The gate which should be opened/closed
    #[arg(value_parser = clap::value_parser!(u8).range(1..=4))]
    pub gate: u8,
    /// The operation to perform on the specified gate
    #[arg(value_enum)]
    pub operation: GateOperation,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GateOperation {
    /// Opens the specifies gate
    Open,
    /// Closes the specified gate
    Close,
}
