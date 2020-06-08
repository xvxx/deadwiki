//! "Native" app using your system's WebKit to browse a local deadwiki
//! instance.

#[cfg(feature = "gui")]
use web_view;

use {
    crate::{app, set_wiki_root, sync},
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

    if wiki_root.is_empty() {
        if let Ok(Some(wiki_root)) = wv.dialog().choose_directory("Wiki Root", "") {
            set_wiki_root(&wiki_root.to_str().unwrap_or("."))?;
            wv.eval(&format!("location.href = \"{}\";", url)).unwrap();
        } else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No Wiki Root Selected.",
            ));
        }
    } else {
        set_wiki_root(wiki_root)?;
    }

    if sync {
        if let Err(e) = sync::start() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Sync Error: {}", e),
            ));
        }
    }

    thread::spawn(move || vial::run!(addr.to_string(), app));

    wv.run().unwrap();
    Ok(())
}
