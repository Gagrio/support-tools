use anyhow::Result;
use chrono::Local;
use clap::Parser;
use tracing::{info, warn};

mod archive;
mod collector;

#[derive(Parser, Debug)]
#[command(
    name = "buttcrack",
    about = "Batch Utility To Transfer Cluster Resources And Configs Kit",
    version
)]
struct Args {
    /// Comma-separated list of host paths to collect
    #[arg(short, long, env = "BC_PATHS", value_delimiter = ',', required = true)]
    paths: Vec<String>,

    /// Output directory for the archive
    #[arg(short, long, env = "BC_OUTPUT", default_value = "/tmp")]
    output: String,

    /// Verbose logging
    #[arg(short, long, env = "BC_VERBOSE", default_value_t = false)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let level = if args.verbose { "info" } else { "warn" };
    tracing_subscriber::fmt()
        .with_env_filter(level)
        .without_time()
        .init();

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();

    info!("Starting BUTTCRACK collection");
    info!("Paths: {:?}", args.paths);
    info!("Output: {}", args.output);

    let entries = collector::collect_paths(&args.paths)?;

    if entries.iter().all(|e| e.is_dir) {
        warn!("No files found in the provided paths - archive will be empty");
    }

    let archive_path = archive::create_archive(&entries, &args.paths, &args.output, &timestamp)?;

    info!("Archive created: {}", archive_path);
    println!("{}", archive_path);

    Ok(())
}
