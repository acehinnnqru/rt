use std::path::Path;

use crate::exec::cmd_stream;

pub fn zoxide(path: &Path) {
    cmd_stream(format!("zoxide add {}", path.to_str().unwrap()).as_str())
}
