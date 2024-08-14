use std::process::{Command, Stdio};

pub fn exec_stream(cmd_str: &str) {
    let mut cmd = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd_str])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd_str)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
    };

    cmd.wait().unwrap();
}
