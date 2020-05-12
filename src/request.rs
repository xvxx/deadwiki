//! Web Request.

use {
    crate::{asset, render},
    ascii::AsciiString,
    atomicwrites::{AllowOverwrite, AtomicFile},
    etag::EntityTag,
    percent_encoding::percent_decode,
    std::{
        fs,
        io::{self, prelude::*},
        path::Path,
        str,
    },
    tiny_http::{
        Header,
        Method::{self, Get, Post},
        Request as TinyRequest, Response,
    },
};

pub struct Request {
    // path to wiki on disk
    root: String,
    // raw TinyHTTP request
    tiny_req: TinyRequest,
}

impl Request {
    /// Make a new Request.
    pub fn new(root: String, req: TinyRequest) -> Request {
        Request {
            root: format!("{}/", root.trim_end_matches("/")),
            tiny_req: req,
        }
    }

    /// HTTP Method
    pub fn method(&self) -> &Method {
        self.tiny_req.method()
    }

    /// URL requested
    pub fn url(&self) -> &str {
        self.tiny_req.url()
    }

    /// Headers for this request.
    pub fn headers(&self) -> &[Header] {
        self.tiny_req.headers()
    }

    /// Provide io::Read
    fn as_reader(&mut self) -> &mut dyn io::Read {
        self.tiny_req.as_reader()
    }

    /// Respond with 404.
    fn respond_404(self) -> Result<(), io::Error> {
        self.respond(Response::from_string("404 Not Found").with_status_code(404))
    }

    /// Respond to this request. Consumes.
    pub fn respond<R>(self, res: Response<R>) -> Result<(), io::Error>
    where
        R: io::Read,
    {
        self.tiny_req.respond(res)
    }

    /// All the wiki pages, in alphabetical order.
    pub fn page_names(&self) -> Vec<String> {
        let mut dirs = vec![];

        for entry in walkdir::WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir()
                && entry.file_name().to_str().unwrap_or("").ends_with(".md")
            {
                let dir = entry.path().display().to_string().replace(&self.root, "");
                let dir = dir.trim_end_matches(".md");
                if !dir.is_empty() {
                    dirs.push(format!("{}", dir));
                }
            }
        }

        dirs.sort();
        dirs
    }

    /// Request handler. Consumes.
    pub fn handle(mut self) -> Result<(), io::Error> {
        // static files
        if self.method() == &Get && self.url().contains('.') {
            let path = dequery(self.url());
            if asset::exists(&path) {
                return self.serve_static_file(&path);
            }
        }

        let (status, body, content_type) = match self.route() {
            Ok(res) => res,
            Err(e) => {
                eprintln!("{}", e);
                (
                    500,
                    format!("<h1>500 Internal Error</h1><pre>{}</pre>", e),
                    "text/html",
                )
            }
        };

        let response = if status == 302 {
            Response::from_data(format!("Redirected to {}", body))
                .with_status_code(status)
                .with_header(header("Location", &body))
        } else {
            Response::from_data(body)
                .with_status_code(status)
                .with_header(header("Content-Type", content_type))
        };

        println!("-> {} {} {}", status, self.method(), self.url());
        self.respond(response)
    }

    /// Route a request. Returns tuple of (Status Code, Body, Content-Type)
    fn route(&mut self) -> Result<(i32, String, &'static str), io::Error> {
        let mut status = 404;
        let mut body = "404 Not Found".to_string();
        let mut content_type = "text/html; charset=utf8";

        let full_url = self.url().to_string();
        let mut parts = full_url.splitn(2, "?");
        let (url, query) = (parts.next().unwrap_or("/"), parts.next().unwrap_or(""));

        match (self.method(), url) {
            (Get, "/") => {
                status = 200;
                body = render::index(&self)?;
            }
            (Get, "/new") => {
                status = 200;
                let mut name = "".to_string();
                if !query.is_empty() {
                    name.push_str(&decode_form_value(&query.replace("name=", "")));
                }

                body = render::layout(
                    "new page",
                    &asset::to_string("new.html")?.replace("{name}", &name),
                    None,
                );
            }
            (Get, "/sleep") => {
                status = 200;
                body = "Zzzzz...".into();
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
            (Get, "/404") => {
                status = 404;
                body = asset::to_string("404.html")?;
            }
            (Post, "/new") => {
                let mut content = String::new();
                self.as_reader().read_to_string(&mut content)?;
                let mut path = String::new();
                let mut mdown = String::new();
                for pair in content.split('&') {
                    let mut parts = pair.splitn(2, '=');
                    let (field, value) = (
                        parts.next().unwrap_or_default(),
                        parts.next().unwrap_or_default(),
                    );
                    match field.as_ref() {
                        "name" => path = render::pathify(&decode_form_value(value)),
                        "markdown" => mdown = decode_form_value(value),
                        _ => {}
                    }
                }
                if !self.page_names().contains(&path.to_lowercase()) {
                    if let Some(disk_path) = render::new_page_path(&path) {
                        if disk_path.contains('/') {
                            if let Some(dir) = Path::new(&disk_path).parent() {
                                fs::create_dir_all(&dir.display().to_string())?;
                            }
                        }
                        let mut file = fs::File::create(disk_path)?;
                        write!(file, "{}", mdown)?;
                        status = 302;
                        body = path.to_string();
                    }
                }
            }
            (Post, path) => {
                if query.is_empty() {
                    status = 404;
                    body = asset::to_string("404.html")?;
                } else {
                    if let Some(disk_path) = render::page_path(path) {
                        let mut content = String::new();
                        self.as_reader().read_to_string(&mut content)?;
                        let mdown = content.split("markdown=").last().unwrap_or("");
                        let af = AtomicFile::new(disk_path, AllowOverwrite);
                        af.write(|f| f.write_all(decode_form_value(mdown).as_bytes()))?;
                        status = 302;
                        body = path.to_string();
                    } else {
                        status = 404;
                        body = asset::to_string("404.html")?;
                    }
                }
            }

            (Get, path) => {
                if let Some(disk_path) = render::page_path(path) {
                    status = 200;
                    if query.is_empty() {
                        body = render::page(self, path).unwrap_or_else(|_| "".into());
                    } else if query == "edit" {
                        body = render::layout(
                            "Edit",
                            &asset::to_string("edit.html")?
                                .replace("{markdown}", &fs::read_to_string(disk_path)?),
                            None,
                        )
                    }
                } else if asset::exists(path) {
                    status = 200;
                    body = asset::to_string(path)?;
                    content_type = get_content_type(path);
                } else {
                    status = 404;
                    body = asset::to_string("404.html")?;
                }
            }

            (x, y) => println!("x: {:?}, y: {:?}", x, y),
        }

        Ok((status, body, content_type))
    }

    /// Serve a static file, doing the header dance with ETag and whatnot.
    fn serve_static_file(self, path: &str) -> Result<(), io::Error> {
        if let Some(file) = asset::Asset::get(&render::pathify(path)) {
            let etag = EntityTag::from_hash(&file);
            if self
                .headers()
                .iter()
                .any(|h| h.field.equiv("If-None-Match") && h.value == etag.tag())
            {
                println!("-> {} {} {}", 304, self.method(), self.url());
                return self.respond(Response::from_data("").with_status_code(304));
            } else {
                println!("-> {} {} {}", 200, self.method(), self.url());
                return self.respond(
                    Response::from_data(file)
                        .with_header(header("ETag", etag.tag()))
                        .with_header(header("Content-Type", get_content_type(&path))),
                );
            }
        }

        self.respond_404()
    }
}

/// Generate a Header for tiny_http.
fn header(field: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header {
        field: field.parse().unwrap(),
        value: AsciiString::from_ascii(value).unwrap_or_else(|_| AsciiString::new()),
    }
}

/// Content type for a file on disk. We only look in `assets/`.
fn get_content_type(path: &str) -> &'static str {
    match path.split('.').last().unwrap_or("?") {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "css" => "text/css; charset=utf8",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}

/// Does what it says.
fn decode_form_value(post: &str) -> String {
    percent_decode(post.as_bytes())
        .decode_utf8_lossy()
        .replace('+', " ")
}

/// Remove the ?query string from a URL.
fn dequery(url: &str) -> String {
    if url.contains('?') {
        url.split('?').next().unwrap_or_else(|| "")
    } else {
        url
    }
    .to_string()
}
