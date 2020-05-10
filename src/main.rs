use deadwiki::web;

fn main() {
    if let Err(e) = web::server("0.0.0.0", 8000) {
        eprintln!("WebServer Error: {}", e);
    }
}
