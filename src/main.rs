use deadwiki::web;

fn main() {
    if let Err(e) = web::run("0.0.0.0", 8000) {
        eprintln!("WebServer Error: {}", e);
    }
}
