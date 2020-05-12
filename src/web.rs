use {
    crate::Request,
    std::{io, sync::Mutex},
    threadpool::ThreadPool,
    tiny_http::Server,
};

/// How many threads to run. Keep it low, this is for personal use!
const MAX_WORKERS: usize = 10;

lazy_static! {
    pub static ref WIKI_ROOT: Mutex<String> = Mutex::new("hello".to_string());
}

/// Run the web server.
pub fn server(root: &str, host: &str, port: usize) -> Result<(), io::Error> {
    if !dir_exists(root) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("can't find {}", root),
        ));
    }

    {
        // if this fails, we want to blow up
        let mut lock = WIKI_ROOT.lock().unwrap();
        *lock = root.to_string();
    }

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
