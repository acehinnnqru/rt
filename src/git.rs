use std::path::Path;

pub fn clone_bare(ssh: &str, target: &Path) {
    let cmd_str = format!("git clone --bare {} {}", ssh, target.to_str().unwrap());

    crate::exec::cmd_stream(&cmd_str)
}

#[cfg(test)]
mod tests {
    use super::clone_bare;

    #[test]
    fn clone_bare_test() {
        let dir = tempdir::TempDir::new("clonetest").unwrap();
        let target = dir.path().join("bare");

        assert!(!target.exists());

        println!("cloning into {}", target.to_str().unwrap());
        clone_bare("git@github.com:acehinnnqru/agrm", &target);

        assert!(target.exists());
        println!("cloned into {}", target.to_str().unwrap());

        dir.close().unwrap();
        println!("cleaned up {}", target.to_str().unwrap());
    }
}
