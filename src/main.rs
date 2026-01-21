//! Voxel Forge - Main Entry Point
//!
//! This is the main executable for the Voxel Forge game engine.

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use voxel_forge::Engine;

fn main() -> anyhow::Result<()> {
    // Initialize logging (default to info level)
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Voxel Forge v{}", voxel_forge::VERSION);

    // Create and run the engine
    let engine = Engine::new()?;
    engine.run()?;

    info!("Voxel Forge shut down successfully");
    Ok(())
}
