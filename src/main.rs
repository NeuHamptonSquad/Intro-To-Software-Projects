use color_eyre::eyre;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(
            EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new("info"))
                .unwrap(),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Hello, world!");
    Ok(())
}
