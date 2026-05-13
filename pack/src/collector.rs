use anyhow::Result;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;
use tracing::{info, warn};
use walkdir::WalkDir;

pub struct CollectedEntry {
    pub absolute_path: PathBuf,
    pub archive_path: String,
    pub is_dir: bool,
}

pub fn collect_paths(paths: &[String]) -> Result<Vec<CollectedEntry>> {
    let mut entries = Vec::new();
    let mut total_files = 0usize;

    for path_str in paths {
        let path = std::path::Path::new(path_str);

        if !path.exists() {
            warn!("Path does not exist, skipping: {}", path_str);
            continue;
        }

        info!("Collecting: {}", path_str);

        for entry in WalkDir::new(path).follow_links(false) {
            match entry {
                Ok(e) => {
                    let ft = e.file_type();
                    let path_display = e.path().display();

                    if ft.is_symlink() {
                        warn!(
                            "Skipping {}: symlink — symlinks are not followed to prevent escaping the intended path or creating loops",
                            path_display
                        );
                        continue;
                    }

                    if ft.is_fifo() {
                        warn!(
                            "Skipping {}: named pipe (FIFO) — reading a FIFO blocks forever waiting for a writer, skipping to avoid hang",
                            path_display
                        );
                        continue;
                    }

                    if ft.is_socket() {
                        warn!(
                            "Skipping {}: Unix socket — opening a socket blocks indefinitely, skipping to avoid hang",
                            path_display
                        );
                        continue;
                    }

                    if ft.is_block_device() {
                        warn!(
                            "Skipping {}: block device — raw block device reads are not supported",
                            path_display
                        );
                        continue;
                    }

                    if ft.is_char_device() {
                        warn!(
                            "Skipping {}: character device — raw character device reads are not supported",
                            path_display
                        );
                        continue;
                    }

                    let is_dir = ft.is_dir();
                    let archive_path = e
                        .path()
                        .to_str()
                        .unwrap_or("")
                        .trim_start_matches('/')
                        .to_string();

                    if !is_dir {
                        total_files += 1;
                    }

                    entries.push(CollectedEntry {
                        absolute_path: e.path().to_path_buf(),
                        archive_path,
                        is_dir,
                    });
                }
                Err(e) => {
                    warn!("Skipping inaccessible entry: {}", e);
                }
            }
        }
    }

    info!("Total files to archive: {}", total_files);
    Ok(entries)
}
