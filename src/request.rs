//! Web Request.

use {
    crate::{asset, render, routes::route, util},
    ascii::AsciiString,
    etag::EntityTag,
    std::{
        collections::HashMap,
        io::{self},
        str,
    },
    tiny_http::{
        Header,
        Method::{self, Get, Post},
        Request as TinyRequest, Response,
    },
};

pub struct Request {
    // raw TinyHTTP request
    tiny_req: TinyRequest,
    // POST/GET params
    params: HashMap<String, String>,
}

impl Request {
    /// Make a new Request.
    pub fn new(req: TinyRequest) -> Request {
        Request {
            tiny_req: req,
            params: HashMap::new(),
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

    /// Return a value in a POST <form> or ?querystring=
    /// Always gives a string. Will be empty if param wasn't sent.
    /// Use `has_param()` to check if it exists.
    pub fn param(&mut self, name: &str) -> &str {
        self.parse_params();
        if let Some(s) = self.params.get(name) {
            s
        } else {
            ""
        }
    }

    /// Has the given param been set?
    pub fn has_param(&mut self, name: &str) -> bool {
        self.parse_params();
        self.params.contains_key(name)
    }

    /// Turn a query string or POST body into a nice and tidy HashMap.
    fn parse_params(&mut self) {
        if !self.params.is_empty() {
            return;
        }

        // temp value
        let mut map = HashMap::new();

        // parse url
        let url = self.url();
        if let Some(start) = url.find('?') {
            parse_query_into_map(&url[start + 1..], &mut map);
        }

        // parse POST body
        if self.method() == &Post {
            let mut content = String::new();
            if let Ok(size) = self.as_reader().read_to_string(&mut content) {
                if size > 0 {
                    parse_query_into_map(&content, &mut map);
                }
            }
        }

        if !map.is_empty() {
            self.params = map;
        }
    }

    /// Provide io::Read
    pub fn as_reader(&mut self) -> &mut dyn io::Read {
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

        for entry in walkdir::WalkDir::new("./")
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_dir()
                && entry.file_name().to_str().unwrap_or("").ends_with(".md")
            {
                let dir = entry.path().display().to_string();
                let dir = dir.trim_start_matches("./").trim_end_matches(".md");
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
            let path = strip_query(self.url()).to_string();
            if asset::exists(&path) {
                return self.serve_static_file(&path);
            }
        }

        let (status, body, content_type) = match route(&mut self) {
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
                        .with_header(header("Content-Type", util::get_content_type(&path))),
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

/// Parses a query string like "name=jimbo&other_data=sure" into a
/// HashMap.
fn parse_query_into_map(params: &str, map: &mut HashMap<String, String>) {
    for kv in params.split('&') {
        let mut parts = kv.splitn(2, '=');
        if let Some(key) = parts.next() {
            if let Some(val) = parts.next() {
                map.insert(key.to_string(), util::decode_form_value(val));
            } else {
                map.insert(key.to_string(), "".to_string());
            }
        }
    }
}

/// Strip the ?querystring from a URL.
fn strip_query(url: &str) -> &str {
    if let Some(idx) = url.find('?') {
        &url[..idx]
    } else {
        url
    }
    .trim_start_matches("static/")
    .trim_start_matches("/static/")
    .trim_start_matches("./static/")
}
