use std::{io::LineWriter, sync::OnceLock};

use godot::{
    classes::{Engine, class_macros::sys::InitLevel},
    prelude::*,
};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod terminal;
mod tracing_godot;

pub use terminal::Terminal;
pub use tracing_godot::Logger;

struct FnafDoubleVisionExtension;

pub(crate) static LOG_SERVER: OnceLock<tracing_godot::LogServer> = OnceLock::new();

#[gdextension]
unsafe impl ExtensionLibrary for FnafDoubleVisionExtension {
    fn on_level_init(level: InitLevel) {
        match level {
            InitLevel::Core => {
                if LOG_SERVER.get().is_none() {
                    color_eyre::config::HookBuilder::new()
                        .theme(color_eyre::config::Theme::default())
                        .install()
                        .unwrap();

                    let log_server = tracing_godot::LogServer::new();
                    LOG_SERVER.set(log_server).unwrap();

                    tracing_subscriber::registry()
                        .with(ErrorLayer::default())
                        .with(
                            EnvFilter::try_from_default_env()
                                .or_else(|_| EnvFilter::try_new("info"))
                                .unwrap(),
                        )
                        .with(
                            tracing_subscriber::fmt::layer()
                                .with_writer(|| LineWriter::new(tracing_godot::LogServerWriter)),
                        )
                        .init();
                }
            }
            InitLevel::Scene => {
                Engine::singleton().register_singleton(
                    &Logger::class_name().to_string_name(),
                    &Logger::new_alloc(),
                );
            }
            InitLevel::Editor => {
                // The server should not run when the game isn't
                if let Some(log_server) = LOG_SERVER.get() {
                    // log_server.join();
                }
            }
            _ => {}
        }
    }

    fn on_level_deinit(level: InitLevel) {
        match level {
            InitLevel::Core => {
                if let Some(log_server) = LOG_SERVER.get() {
                    log_server.join();
                }
            }
            InitLevel::Scene => {
                let mut engine = Engine::singleton();
                let singleton_name = &Logger::class_name().to_string_name();
                if let Some(logger_singleton) = engine.get_singleton(singleton_name) {
                    engine.unregister_singleton(singleton_name);
                    logger_singleton.free();
                }
            }
            _ => {}
        }
    }
}
