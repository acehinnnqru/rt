use std::path::PathBuf;

#[cfg(target_family = "windows")]
fn default_tmp_dir() -> PathBuf {
    PathBuf::from(std::env::var("TEMP").unwrap()).join("agrm")
}

#[cfg(target_family = "unix")]
fn default_tmp_dir() -> PathBuf {
    PathBuf::from("/tmp").join("agrm")
}
