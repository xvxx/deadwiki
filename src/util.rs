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
