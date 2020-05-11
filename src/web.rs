use {crate::Request, std::io, threadpool::ThreadPool, tiny_http::Server};

/// How many threads to run. Keep it low, this is for personal use!
const MAX_WORKERS: usize = 10;

/// Run the web server.
pub fn server(root: &str, host: &str, port: usize) -> Result<(), io::Error> {
    let pool = ThreadPool::new(MAX_WORKERS);
    let addr = format!("{}:{}", host, port);
    let server = Server::http(&addr).expect("Server Error: ");
    println!("-> deadwiki serving {} at http://{}", root, addr);

    for tiny_req in server.incoming_requests() {
        let root = root.to_string();
        pool.execute(move || {
            let req = Request::new(root, tiny_req);
            if let Err(e) = req.handle() {
                eprintln!("!> {}", e);
            }
        });
    }

    Ok(())
}
