use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ScanStats {
    pub total_bytes: u64,
    pub file_count: u64,
    pub dir_count: u64,
    pub skipped_count: u64,
}

struct AtomicStats {
    total_bytes: AtomicU64,
    file_count: AtomicU64,
    dir_count: AtomicU64,
    skipped_count: AtomicU64,
}

impl AtomicStats {
    fn new() -> Self {
        Self {
            total_bytes: AtomicU64::new(0),
            file_count: AtomicU64::new(0),
            dir_count: AtomicU64::new(0),
            skipped_count: AtomicU64::new(0),
        }
    }

    fn snapshot(&self) -> ScanStats {
        ScanStats {
            total_bytes: self.total_bytes.load(Ordering::Relaxed),
            file_count: self.file_count.load(Ordering::Relaxed),
            dir_count: self.dir_count.load(Ordering::Relaxed),
            skipped_count: self.skipped_count.load(Ordering::Relaxed),
        }
    }
}

/// Recursively scans a directory using Rayon for parallel traversal of subtrees.
/// Never follows symlinks or junctions to avoid cycles.
/// On any access error (permission denied, missing file), increments `skipped_count` and continues.
pub fn scan_directory(root: &Path, cancel: &AtomicBool) -> ScanStats {
    let stats = AtomicStats::new();
    scan_dir_internal(root, cancel, &stats);
    stats.snapshot()
}

fn scan_dir_internal(dir: &Path, cancel: &AtomicBool, stats: &AtomicStats) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(iter) => iter,
        Err(_) => {
            stats.skipped_count.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };

    let mut subdirs = Vec::new();

    for entry_res in entries {
        if cancel.load(Ordering::Relaxed) {
            return;
        }

        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => {
                stats.skipped_count.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let path = entry.path();
        // Check symlink_metadata to avoid following junctions/symlinks on Windows
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => {
                stats.skipped_count.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        };

        let file_type = meta.file_type();
        if file_type.is_symlink() {
            // Reparse point / Symlink: do NOT follow, only count its own size
            stats.file_count.fetch_add(1, Ordering::Relaxed);
            stats.total_bytes.fetch_add(meta.len(), Ordering::Relaxed);
        } else if file_type.is_dir() {
            stats.dir_count.fetch_add(1, Ordering::Relaxed);
            subdirs.push(path);
        } else {
            stats.file_count.fetch_add(1, Ordering::Relaxed);
            stats.total_bytes.fetch_add(meta.len(), Ordering::Relaxed);
        }
    }

    if cancel.load(Ordering::Relaxed) {
        return;
    }

    // Parallelize subdirectories across Rayon's thread pool
    if subdirs.len() == 1 {
        scan_dir_internal(&subdirs[0], cancel, stats);
    } else if subdirs.len() > 1 {
        subdirs.into_par_iter().for_each(|subdir| {
            scan_dir_internal(&subdir, cancel, stats);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_scan_directory_basic() {
        let temp_dir = std::env::temp_dir().join("better_folder_size_test_basic");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let sub1 = temp_dir.join("sub1");
        fs::create_dir_all(&sub1).unwrap();

        let sub2 = temp_dir.join("sub2");
        fs::create_dir_all(&sub2).unwrap();

        let file1 = temp_dir.join("file1.txt");
        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(b"Hello 10B!").unwrap(); // 10 bytes

        let file2 = sub1.join("file2.bin");
        let mut f2 = File::create(&file2).unwrap();
        f2.write_all(&[0u8; 100]).unwrap(); // 100 bytes

        let cancel = AtomicBool::new(false);
        let stats = scan_directory(&temp_dir, &cancel);

        assert_eq!(stats.file_count, 2);
        assert_eq!(stats.dir_count, 2);
        assert_eq!(stats.total_bytes, 110);
        assert_eq!(stats.skipped_count, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_scan_cancellation() {
        let temp_dir = std::env::temp_dir().join("better_folder_size_test_cancel");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let cancel = AtomicBool::new(true); // pre-canceled
        let stats = scan_directory(&temp_dir, &cancel);

        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.total_bytes, 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_benchmark_real_folder() {
        let cargo_dir = dirs_or_fallback();
        if cargo_dir.exists() {
            let cancel = AtomicBool::new(false);
            let start = std::time::Instant::now();
            let stats = scan_directory(&cargo_dir, &cancel);
            let elapsed = start.elapsed();
            println!(
                "\n[BENCHMARK] Scanned {:?}: {} files, {} dirs, {} bytes in {:?}",
                cargo_dir, stats.file_count, stats.dir_count, stats.total_bytes, elapsed
            );
        }
    }

    fn dirs_or_fallback() -> std::path::PathBuf {
        std::env::var("USERPROFILE")
            .map(|u| std::path::PathBuf::from(u).join(".cargo"))
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
    }
}
