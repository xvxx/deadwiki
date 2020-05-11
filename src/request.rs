//! Web Request.
use {
    ascii::AsciiString,
    atomicwrites::{AllowOverwrite, AtomicFile},
    etag::EntityTag,
    percent_encoding::percent_decode,
    pulldown_cmark as markdown,
    rust_embed::RustEmbed,
    std::{
        fs,
        io::{self, prelude::*},
        os::unix::fs::PermissionsExt,
        path::Path,
        process::Command,
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

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Asset;

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

    /// Path of wiki page on disk, if it exists.
    /// Ex: page_path("Welcome") -> "wiki/welcome.md"
    fn page_path(&self, path: &str) -> Option<String> {
        let path = self.page_disk_path(path);
        if Path::new(&path).exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Returns a path on disk to a new wiki page.
    /// Nothing if the page already exists.
    fn new_page_path(&self, path: &str) -> Option<String> {
        if self.page_path(path).is_none() {
            Some(self.page_disk_path(path))
        } else {
            None
        }
    }

    /// Returns a wiki path on disk, regardless of whether it exists.
    fn page_disk_path(&self, path: &str) -> String {
        format!("{}/{}.md", self.root, pathify(path))
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
            if asset_exists(&self.url()) {
                let path = self.url().to_string();
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

    /// Render a wiki page to a fully loaded HTML string.
    fn render_page(&self, path: &str) -> Result<String, io::Error> {
        let raw = path.ends_with(".md");
        let path = if raw {
            path.trim_end_matches(".md")
        } else {
            path
        };
        let title = wiki_path_to_title(path);
        if let Some(path) = self.page_path(path) {
            let html = if is_executable(&path) {
                shell(&path, &[]).unwrap_or_else(|e| e.to_string())
            } else {
                fs::read_to_string(path).unwrap_or_else(|_| "".into())
            };
            Ok(if raw {
                format!("<pre>{}</pre>", html)
            } else {
                self.render_with_layout(&title, &self.markdown_to_html(&html), Some(&nav()?))
            })
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} not found", path),
            ))
        }
    }

    /// Renders a chunk of HTML surrounded by `assets/layout.html`.
    fn render_with_layout(&self, title: &str, body: &str, nav: Option<&str>) -> String {
        if asset_exists("layout.html") {
            asset_to_string("layout.html")
                .unwrap_or_else(|_| "".into())
                .replace("{title}", title)
                .replace("{body}", body)
                .replace("{nav}", nav.unwrap_or(""))
        } else {
            body.to_string()
        }
    }

    /// Convert raw Markdown into HTML.
    fn markdown_to_html(&self, md: &str) -> String {
        let mut options = markdown::Options::empty();
        options.insert(markdown::Options::ENABLE_TASKLISTS);
        options.insert(markdown::Options::ENABLE_FOOTNOTES);

        // are we parsing a wiki link like [Help] or [Solar Power]?
        let mut wiki_link = false;
        // if we are, store the text between [ and ]
        let mut wiki_link_text = String::new();

        let parser = markdown::Parser::new_ext(&md, options).map(|event| match event {
            markdown::Event::Text(text) => {
                if text.as_ref() == "[" && !wiki_link {
                    wiki_link = true;
                    markdown::Event::Text("".into())
                } else if text.as_ref() == "]" && wiki_link {
                    wiki_link = false;
                    let page_name = wiki_link_text.to_lowercase().replace(" ", "_");
                    let link_text = wiki_link_text.clone();
                    wiki_link_text.clear();
                    let page_exists = self.page_names().contains(&page_name);
                    let (link_class, link_href) = if page_exists {
                        ("", format!("/{}", page_name))
                    } else {
                        ("new", format!("/new?name={}", page_name))
                    };
                    markdown::Event::Html(
                        format!(
                            r#"<a href="{}" class="{}">{}</a>"#,
                            link_href, link_class, link_text
                        )
                        .into(),
                    )
                } else if wiki_link {
                    wiki_link_text.push_str(&text);
                    markdown::Event::Text("".into())
                } else {
                    let linked = autolink::auto_link(&text, &[]);
                    if linked.len() == text.len() {
                        markdown::Event::Text(text)
                    } else {
                        markdown::Event::Html(linked.into())
                    }
                }
            }
            _ => event,
        });

        let mut html_output = String::with_capacity(md.len() * 3 / 2);
        markdown::html::push_html(&mut html_output, parser);
        html_output
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
                body = self.render_with_layout(
                    "deadwiki",
                    &format!(
                        "<p><a href='/new'>new</a></p><h1>deadwiki</h1>\n<ul>\n{}</ul>\n<hr>",
                        self.page_names()
                            .iter()
                            .map(|name| format!(
                                "  <li><a href='{}'>{}</a></li>\n",
                                name,
                                wiki_path_to_title(name)
                            ))
                            .collect::<String>()
                    ),
                    None,
                );
            }
            (Get, "/new") => {
                status = 200;
                let mut name = "".to_string();
                if !query.is_empty() {
                    name.push_str(&decode_form_value(&query.replace("name=", "")));
                }

                body = self.render_with_layout(
                    "new page",
                    &asset_to_string("new.html")?.replace("{name}", &name),
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
                body = asset_to_string("404.html")?;
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
                        "name" => path = pathify(&decode_form_value(value)),
                        "markdown" => mdown = decode_form_value(value),
                        _ => {}
                    }
                }
                if !self.page_names().contains(&path.to_lowercase()) {
                    if let Some(disk_path) = self.new_page_path(&path) {
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
                    body = asset_to_string("404.html")?;
                } else {
                    if let Some(disk_path) = self.page_path(path) {
                        let mut content = String::new();
                        self.as_reader().read_to_string(&mut content)?;
                        let mdown = content.split("markdown=").last().unwrap_or("");
                        let af = AtomicFile::new(disk_path, AllowOverwrite);
                        af.write(|f| f.write_all(decode_form_value(mdown).as_bytes()))?;
                        status = 302;
                        body = path.to_string();
                    } else {
                        status = 404;
                        body = asset_to_string("404.html")?;
                    }
                }
            }

            (Get, path) => {
                if let Some(disk_path) = self.page_path(path) {
                    status = 200;
                    if query.is_empty() {
                        body = self.render_page(path).unwrap_or_else(|_| "".into());
                    } else if query == "edit" {
                        body = self.render_with_layout(
                            "Edit",
                            &asset_to_string("edit.html")?
                                .replace("{markdown}", &fs::read_to_string(disk_path)?),
                            None,
                        )
                    }
                } else if asset_exists(path) {
                    status = 200;
                    body = asset_to_string(path)?;
                    content_type = get_content_type(path);
                } else {
                    status = 404;
                    body = asset_to_string("404.html")?;
                }
            }

            (x, y) => println!("x: {:?}, y: {:?}", x, y),
        }

        Ok((status, body, content_type))
    }

    /// Serve a static file, doing the header dance with ETag and whatnot.
    fn serve_static_file(self, path: &str) -> Result<(), io::Error> {
        if let Some(file) = Asset::get(&pathify(path)) {
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

/// Does the asset exist on disk?
fn asset_exists(path: &str) -> bool {
    Asset::get(&pathify(path)).is_some()
}

/// like fs::read_to_string() but with an asset.
fn asset_to_string(path: &str) -> Result<String, io::Error> {
    if let Some(asset) = Asset::get(&path) {
        if let Ok(utf8) = str::from_utf8(asset.as_ref()) {
            return Ok(utf8.to_string());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} not found", path),
    ))
}

/// Convert a wiki or asset name to a path-friendly string.
/// Ex: "Test Results" -> "test_results"
fn pathify(path: &str) -> String {
    path.to_lowercase()
        .trim_start_matches('/')
        .replace("..", ".")
        .replace(" ", "_")
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

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap_or('?').to_uppercase(),
        &s.chars().skip(1).collect::<String>()
    )
}

/// some_page -> Some Page
fn wiki_path_to_title(path: &str) -> String {
    path.trim_start_matches('/')
        .split('_')
        .map(|part| {
            if part.contains('/') {
                let mut parts = part.split('/').rev();
                let last = parts.next().unwrap_or("?");
                format!(
                    "{}/{}",
                    parts.rev().collect::<Vec<_>>().join("/"),
                    capitalize(last)
                )
            } else {
                capitalize(&part)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Is the file at the given path `chmod +x`?
fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Run a script and return its output.
fn shell(path: &str, args: &[&str]) -> Result<String, io::Error> {
    let output = Command::new(path).args(args).output()?;
    let out = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };
    match str::from_utf8(&out) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
    }
}

/// Return the <nav> for a page
fn nav() -> Result<String, io::Error> {
    asset_to_string("nav.html")
}
