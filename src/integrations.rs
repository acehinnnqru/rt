use std::path::Path;

use crate::cmd::exec_stream;

pub fn zoxide(path: &Path) {
    exec_stream(format!("zoxide add {}", path.to_str().unwrap()).as_str())
}
