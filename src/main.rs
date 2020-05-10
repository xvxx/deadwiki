use ascii::AsciiString;
use std::{fs, path::Path};
use tiny_http::{Method::Get, Response, Server};

fn main() -> Result<(), std::io::Error> {
    let host = "0.0.0.0";
    let port = 8000;
    let server = Server::http(format!("{}:{}", host, port)).unwrap();

    for request in server.incoming_requests() {
        let mut body = "404 Not Found".to_string();
        let mut status = 404;
        println!("{} {}", request.method(), request.url());
        match (request.method(), request.url()) {
            (Get, "/dog") => {
                status = 200;
                body = "woof woof".into();
            }
            (Get, "/cat") => {
                status = 200;
                body = "meow".into();
            }
            (Get, "/") | (Get, "/index.html") => {
                status = 200;
                body = fs::read_to_string("index.html")?;
            }
            (Get, path) => {
                let path = format!(".{}", path.replace("..", "."));
                if Path::new(&path).exists() {
                    status = 200;
                    body = fs::read_to_string(path)?;
                } else {
                    status = 404;
                }
            }
            (x, y) => println!("x: {:?}, y: {:?}", x, y),
        }

        let mut path = Path::new(request.url());
        if status == 404 {
            path = Path::new("404.html");
            body = fs::read_to_string("404.html")?;
        }
        let response = Response::from_string(body).with_status_code(status);
        let response = response.with_header(tiny_http::Header {
            field: "Content-Type".parse().unwrap(),
            value: AsciiString::from_ascii(get_content_type(&path)).unwrap(),
        });

        if let Err(e) = request.respond(response) {
            eprintln!(">> {:?}", e);
        }
    }

    Ok(())
}

fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}
