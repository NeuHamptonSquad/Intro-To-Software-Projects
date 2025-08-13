use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct InitCommand {
    #[arg(value_enum)]
    /// The component of the system to initialize
    pub component: Component,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
pub enum Component {
    Vent,
    Camera,
    Audio,
}
