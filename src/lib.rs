#[macro_use]
extern crate lazy_static;

pub mod asset;
pub mod render;
pub mod request;
pub mod sync;
pub mod web;

pub use request::Request;

lazy_static! {
    pub static ref WIKI_ROOT: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
}

/// Path to our deadwiki.
pub fn wiki_root() -> String {
    WIKI_ROOT.lock().unwrap().clone()
}

/// Use sparingly! Set the wiki root.
/// panic! on fail
pub fn set_wiki_root(path: &str) -> Result<(), std::io::Error> {
    if !dir_exists(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} isn't a directory", path),
        ));
    }

    // set current dir to wiki
    std::env::set_current_dir(path).expect("couldn't change working dir");

    // if this fails, we want to blow up
    let mut lock = WIKI_ROOT.lock().unwrap();
    *lock = path.to_string();

    Ok(())
}

/// Does this directory exist?
fn dir_exists(path: &str) -> bool {
    if std::path::Path::new(path).exists() {
        if let Ok(file) = std::fs::File::open(path) {
            if let Ok(meta) = file.metadata() {
                return meta.is_dir();
            }
        }
    }
    false
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
