use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::config::Config;

pub async fn run(config: &Config) -> Result<()> {
    let root = Path::new(&config.root);

    if !root.exists() {
        bail!("root directory does not exist: {}", root.display());
    }

    println!("{}", root.display());
    print_tree(root, "", 0)?;
    Ok(())
}

fn print_tree(dir: &Path, prefix: &str, depth: u32) -> Result<()> {
    if depth >= 3 {
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter(|e| {
            !e.file_name()
                .to_string_lossy()
                .starts_with('.')
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let is_last = i == count - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let name = entry.file_name();
        println!("{prefix}{connector}{}", name.to_string_lossy());

        let child_prefix = format!("{prefix}{}", if is_last { "    " } else { "│   " });
        print_tree(&entry.path(), &child_prefix, depth + 1)?;
    }

    Ok(())
}
