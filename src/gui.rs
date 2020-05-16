#[cfg(feature = "gui")]
use web_view;

use {
    crate::web,
    std::{io, thread},
};

type Result<T> = std::result::Result<T, io::Error>;

/// Start a web server and launch the GUI.
pub fn run(host: &str, port: usize) -> Result<()> {
    let host = host.to_string();
    let url = format!("http://{}:{}", host, port);
    thread::spawn(move || web::server(&host, port));

    start(&url)
}

/// Start the GUI.
fn start(url: &str) -> Result<()> {
    web_view::builder()
        .title("deadwiki")
        .content(web_view::Content::Url(url))
        .size(1024, 768)
        .resizable(true)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
    Ok(())
}
