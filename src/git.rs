use std::{path::Path, process::Command};

pub fn clone_bare(ssh: &str, target: &Path) {
    let cmd_str = format!("git clone --bare {} {}", ssh, target.to_str().unwrap());

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd_str.as_str()])
            .output()
            .expect("failed to execute clone process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .output()
            .expect("failed to execute clone process")
    };

    println!();
    println!("stdout:");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    println!("stderr:");
    println!("{}", String::from_utf8_lossy(&output.stderr));
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
