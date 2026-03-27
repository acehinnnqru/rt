use std::env;
use std::path::PathBuf;

pub mod autojump;
pub mod zoxide;

pub fn command_exists(command: &str) -> bool {
    let Some(path_var) = env::var_os("PATH") else {
        return false;
    };

    env::split_paths(&path_var)
        .map(|dir| dir.join(command))
        .any(|candidate: PathBuf| candidate.is_file())
}
