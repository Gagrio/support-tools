use crate::collector::CollectedEntry;
use anyhow::{Context, Result};
use std::fs;
use tracing::info;

pub fn create_archive(
    entries: &[CollectedEntry],
    source_paths: &[String],
    output_dir: &str,
    timestamp: &str,
) -> Result<String> {
    let archive_name = format!("{}/buttcrack_logs_{}.tar.gz", output_dir, timestamp);
    let top_dir = format!("buttcrack_logs_{}", timestamp);

    info!("Creating archive: {}", archive_name);

    let tar_gz = fs::File::create(&archive_name).context("Failed to create archive file")?;
    let enc = flate2::write::GzEncoder::new(tar_gz, flate2::Compression::default());
    let mut tar = tar::Builder::new(enc);

    let summary = build_summary(entries, source_paths, timestamp);
    let summary_bytes = summary.as_bytes();
    let mut header = tar::Header::new_gnu();
    header.set_path(format!("{}/collection-summary.yaml", top_dir))?;
    header.set_size(summary_bytes.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    tar.append(&header, summary_bytes)
        .context("Failed to add summary to archive")?;

    for entry in entries {
        let archive_path = format!("{}/{}", top_dir, entry.archive_path);

        if entry.is_dir {
            let mut header = tar::Header::new_gnu();
            header.set_path(&archive_path)?;
            header.set_entry_type(tar::EntryType::Directory);
            header.set_mode(0o755);
            header.set_size(0);
            header.set_cksum();
            tar.append(&header, &[][..])
                .context(format!("Failed to add directory: {}", archive_path))?;
        } else {
            tar.append_path_with_name(&entry.absolute_path, &archive_path)
                .context(format!(
                    "Failed to add file: {}",
                    entry.absolute_path.display()
                ))?;
        }
    }

    tar.finish().context("Failed to finalize archive")?;

    info!("Archive finalized: {}", archive_name);
    Ok(archive_name)
}

fn build_summary(entries: &[CollectedEntry], source_paths: &[String], timestamp: &str) -> String {
    let file_count = entries.iter().filter(|e| !e.is_dir).count();
    let dir_count = entries.iter().filter(|e| e.is_dir).count();

    let paths_yaml = source_paths
        .iter()
        .map(|p| format!("  - {}", p))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "collection_info:\n  timestamp: \"{}\"\n  tool: buttcrack\n  version: \"{}\"\n\npaths_collected:\n{}\n\ncollection_summary:\n  total_files: {}\n  total_directories: {}\n",
        timestamp,
        env!("CARGO_PKG_VERSION"),
        paths_yaml,
        file_count,
        dir_count,
    )
}
