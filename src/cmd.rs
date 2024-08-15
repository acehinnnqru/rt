use std::process::Command;

pub fn exec_stream(cmd_str: &str) {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg(cmd_str);
        cmd
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(cmd_str);
        cmd
    };

    match cmd.spawn() {
        Ok(mut p) => p.wait().map(|status| if !status.success() {
            panic!("\nexit error:\n {}", status);
        }).unwrap(),
        Err(err) => panic!("{}", err),
    };
}
