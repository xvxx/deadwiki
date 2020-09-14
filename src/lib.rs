#[macro_use]
extern crate lazy_static;

#[macro_use]
pub mod util;
pub mod app;
pub mod db;
#[cfg(feature = "gui")]
pub mod gui;
pub mod helper;
pub mod markdown;
mod page;
pub mod render;
pub mod sync;

pub use page::Page;

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
    let path = if path.contains('~') {
        match std::env::var("HOME") {
            Ok(home) => path.replace('~', &home),
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "No $HOME env var! Can't decode `~`",
                ))
            }
        }
    } else {
        path.to_string()
    };

    // ensure there's always one trailing /
    let path = format!("{}/", path.trim_end_matches('/'));

    if !dir_exists(&path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("{} isn't a directory", path),
        ));
    }

    // if this fails, we want to blow up
    let mut lock = WIKI_ROOT.lock().unwrap();
    *lock = path;

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
