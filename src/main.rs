#[cfg(feature = "profiling")]
use chrono::Utc;
use std::env;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub mod api;
pub mod db;
pub mod eth_client;
pub mod indexer;
pub mod types;

#[cfg(feature = "profiling")]
fn start_profiling() -> pprof::Result<()> {
    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(1000)
        .blocklist(&["libc", "libgcc", "pthread", "vdso"])
        .build()?;

    std::thread::spawn(move || {
        loop {
            let report_dir = "report";
            if let Err(e) = std::fs::create_dir_all(report_dir) {
                tracing::error!("Failed to create report directory: {}", e);
                // Continue without saving if directory creation fails
                std::thread::sleep(std::time::Duration::from_secs(60));
                continue;
            }

            let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
            let filename = format!("{}/{}-flamegraph.svg", report_dir, timestamp);

            match guard.report().build() {
                Ok(report) => match std::fs::File::create(&filename) {
                    Ok(mut file) => {
                        if let Err(e) = report.flamegraph(&mut file) {
                            tracing::error!("Failed to write flamegraph to file: {}", e);
                        } else {
                            tracing::info!("Flamegraph generated: {}", filename);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create flamegraph file {}: {}", filename, e);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to build profiling report: {}", e);
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    });

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    #[cfg(feature = "profiling")]
    start_profiling()?;

    let rpc_url = env::var("JSON_RPC_API_KEY")
        .expect("JSON_RPC_API_KEY must be set. You can set it in .env file");
    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL must be set. You can set it in .env file");
    indexer::start(rpc_url, &database_url).await?;
    Ok(())
}
