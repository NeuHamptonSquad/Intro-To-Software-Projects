use std::{
    ops::{Add, Div, Mul, RangeInclusive, Sub},
    sync::OnceLock,
};

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
                let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::new().into_hooks();
                let godot_panic_hook = std::panic::take_hook();
                std::panic::set_hook(Box::new(move |panic_info| {
                    if let Some(log_server) = LOG_SERVER.get() {
                        log_server.send(tracing_godot::LogServerEvent::Log(
                            format!("{}", panic_hook.panic_report(panic_info)).into(),
                        ));
                    }
                    (godot_panic_hook)(panic_info)
                }));
                eyre_hook.install().unwrap();

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
                            .with_writer(tracing_godot::LogServerMakeWriter),
                    )
                    .init();
            }
            InitLevel::Scene => {
                Engine::singleton().register_singleton(
                    &Logger::class_name().to_string_name(),
                    &Logger::new_alloc(),
                );
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

// Useful traits, and lerp function from `emath`
// https://docs.rs/emath/0.32.1/src/emath/lib.rs.html#106-113

pub trait One {
    const ONE: Self;
}

impl One for f32 {
    const ONE: Self = 1.0;
}

impl One for f64 {
    const ONE: Self = 1.0;
}

pub trait Real:
    Copy
    + PartialEq
    + PartialOrd
    + One
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Mul<Self, Output = Self>
    + Div<Self, Output = Self>
{
}

impl Real for f32 {}

impl Real for f64 {}

#[inline(always)]
pub fn lerp<R, T>(range: impl Into<RangeInclusive<R>>, t: T) -> R
where
    T: Real + Mul<R, Output = R>,
    R: Copy + Add<R, Output = R>,
{
    let range = range.into();
    (T::ONE - t) * *range.start() + t * *range.end()
}

#[inline]
pub fn inverse_lerp<R>(range: RangeInclusive<R>, value: R) -> Option<R>
where
    R: Copy + PartialEq + Sub<R, Output = R> + Div<R, Output = R>,
{
    let min = *range.start();
    let max = *range.end();
    if min == max {
        None
    } else {
        Some((value - min) / (max - min))
    }
}
