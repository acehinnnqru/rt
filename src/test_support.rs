//! Helpers shared by the unit tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Throwaway directory that is removed once it is dropped.
pub struct TempDir(PathBuf);

impl TempDir {
    /// Creates a uniquely named directory under the system temp directory.
    pub fn new(prefix: &str) -> Self {
        let unique = format!(
            "rt-{prefix}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).expect("should create temp directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Creates `relative` below the temporary directory and returns it.
    pub fn dir(&self, relative: &str) -> PathBuf {
        let path = self.0.join(relative);
        std::fs::create_dir_all(&path).expect("should create temp subdirectory");
        path
    }

    /// Writes `contents` to `relative`, creating parent directories.
    pub fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("should create temp parent directory");
        }
        std::fs::write(&path, contents).expect("should write temp file");
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
