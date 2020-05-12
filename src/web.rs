use {
    crate::{wiki_root, Request},
    std::io,
    threadpool::ThreadPool,
    tiny_http::Server,
};

/// How many threads to run. Keep it low, this is for personal use!
const MAX_WORKERS: usize = 10;

/// Run the web server.
pub fn server(host: &str, port: usize) -> Result<(), io::Error> {
    let pool = ThreadPool::new(MAX_WORKERS);
    let addr = format!("{}:{}", host, port);
    let server = Server::http(&addr).expect("Server Error: ");
    println!("-> deadwiki serving {} at http://{}", wiki_root(), addr);

    for tiny_req in server.incoming_requests() {
        pool.execute(move || {
            let req = Request::new(tiny_req);
            if let Err(e) = req.handle() {
                eprintln!("!> {}", e);
            }
        });
    }

    Ok(())
}
