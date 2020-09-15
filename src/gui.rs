//! "Native" app using your system's WebKit to browse a local deadwiki
//! instance.

#[cfg(feature = "gui")]
use web_view;

use {
    crate::{app, db, sync},
    std::{io, thread},
};

type Result<T> = std::result::Result<T, io::Error>;

/// Start a web server and launch the GUI.
pub fn run(host: &str, port: usize, wiki_root: &str, sync: bool) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    let url = format!("http://{}", addr);

    let mut wv = web_view::builder()
        .title("deadwiki")
        .content(web_view::Content::Url(&url))
        .size(1024, 768)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .build()
        .unwrap();

    let root = if wiki_root.is_empty() {
        if let Ok(Some(root)) = wv.dialog().choose_directory("Wiki Root", "") {
            wv.eval(&format!("location.href = \"{}\";", url)).unwrap();
            root.to_str().unwrap_or(".")
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No Wiki Root Selected.",
            ));
        }
    } else {
        wiki_root
    };

    if sync {
        if let Err(e) = sync::start() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Sync Error: {}", e),
            ));
        }
    }

    let db = db::DB::new(deadwiki::wiki_root());
    vial::use_state!(db);
    thread::spawn(move || vial::run!(addr.to_string(), app));

    wv.run().unwrap();
    Ok(())
}
