use godot::prelude::*;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod terminal;
mod tracing_godot;

pub use terminal::Terminal;

struct FnafDoubleVisionExtension;

#[gdextension]
unsafe impl ExtensionLibrary for FnafDoubleVisionExtension {
    fn on_level_init(level: InitLevel) {
        if matches!(level, InitLevel::Core) {
            color_eyre::config::HookBuilder::new()
                .theme(color_eyre::config::Theme::default())
                .install()
                .unwrap();

            tracing_subscriber::registry()
                .with(ErrorLayer::default())
                .with(
                    EnvFilter::try_from_default_env()
                        .or_else(|_| EnvFilter::try_new("info"))
                        .unwrap(),
                )
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_writer(tracing_godot::GodotMakeWriter)
                        .with_ansi(false),
                )
                .init();
        }
    }
}
