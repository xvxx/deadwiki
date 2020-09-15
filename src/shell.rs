/// Run a shell command, ex: shell!("grep -R '#deadwiki' {}", dir())
macro_rules! shell {
    ($cmd:expr) => {{
        #[cfg(debug_assertions)]
        println!("sh> {}", $cmd);
        crate::shell::shell("sh", &["-c", $cmd.as_ref()])
    }};
    ($cmd:expr, $($arg:tt)+) => {
        shell!(format!($cmd, $($arg)+));
    };
}

/// Run a script and return its output.
pub fn shell(path: &str, args: &[&str]) -> Result<String, std::io::Error> {
    let output = std::process::Command::new(path).args(args).output()?;
    let out = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };
    match std::str::from_utf8(&out) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )),
    }
}
