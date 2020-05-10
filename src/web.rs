use ascii::AsciiString;
use pulldown_cmark as markdown;
use threadpool::ThreadPool;
use tiny_http::{Method::Get, Request, Response, Server};

use std::{fs, io, os::unix::fs::PermissionsExt, path::Path, process::Command, str};

/// How many threads to run. Keep it low, this is for personal use!
const MAX_WORKERS: usize = 10;

/// Run the web server.
pub fn run(host: &str, port: usize) -> Result<(), io::Error> {
    let addr = format!("{}:{}", host, port);

    println!("-> running at http://{}", addr);
    let server = Server::http(addr).unwrap();
    let pool = ThreadPool::new(MAX_WORKERS);

    for request in server.incoming_requests() {
        pool.execute(move || {
            if let Err(e) = handle(request) {
                eprintln!("!> {}", e);
            }
        });
    }

    Ok(())
}

/// Handle a single request.
fn handle(req: Request) -> Result<(), io::Error> {
    let mut body = "404 Not Found".to_string();
    let mut status = 404;
    let mut content_type = "text/html; charset=utf8";

    match (req.method(), req.url()) {
        (Get, "/") => {
            status = 200;
            body = fs::read_to_string("web/index.html")?;
        }
        (Get, "/sleep") => {
            status = 200;
            body = "Zzzzz...".into();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        (Get, "/404") => {
            status = 404;
            body = fs::read_to_string("web/404.html")?;
        }

        (Get, path) => {
            if let Some(html) = render_wiki(path) {
                status = 200;
                body = html;
            } else if let Some(web_path) = web_path(path) {
                status = 200;
                body = fs::read_to_string(web_path)?;
                content_type = get_content_type(path).unwrap_or("text/plain");
            } else {
                status = 404;
                body = fs::read_to_string("web/404.html")?;
            }
        }

        (x, y) => println!("x: {:?}, y: {:?}", x, y),
    }

    let response = Response::from_data(body).with_status_code(status);

    let response = response.with_header(tiny_http::Header {
        field: "Content-Type".parse().unwrap(),
        value: AsciiString::from_ascii(content_type).unwrap(),
    });

    println!("-> {} {} {}", status, req.method(), req.url());
    req.respond(response)
}

/// Render a wiki page to a fully loaded HTML string.
/// Wiki pages are stored in the `wiki/` directory as `.md` files.
fn render_wiki(path: &str) -> Option<String> {
    let raw = path.ends_with(".md");
    let path = if raw {
        path.trim_end_matches(".md")
    } else {
        path
    };
    if let Some(path) = wiki_path(path) {
        let html = if is_executable(&path) {
            shell(&path, &[]).unwrap_or_else(|e| e.to_string())
        } else {
            fs::read_to_string(path).unwrap_or_else(|_| "".into())
        };
        Some(if raw {
            format!("<pre>{}</pre>", html)
        } else {
            render_with_layout("Title", &markdown_to_html(&html))
        })
    } else {
        None
    }
}

/// Renders a chunk of HTML surrounded by `web/layout.html`.
fn render_with_layout(title: &str, body: &str) -> String {
    if let Some(layout) = web_path("layout.html") {
        fs::read_to_string(layout)
            .unwrap_or_else(|_| "".into())
            .replace("{title}", title)
            .replace("{body}", body)
    } else {
        body.to_string()
    }
}

/// Lowercase names of all the wiki pages.
fn wiki_page_names() -> Vec<String> {
    let mut dirs = vec![];

    if let Ok(entries) = fs::read_dir("./wiki") {
        for dir in entries {
            if let Ok(dir) = dir {
                dirs.push(
                    dir.path()
                        .to_str()
                        .unwrap_or_else(|| "?")
                        .trim_start_matches("./wiki/")
                        .trim_end_matches(".md")
                        .to_string(),
                )
            }
        }
    }

    dirs
}

/// [Help] => <a href="/help">Help</a>
fn autolink_pages(html: &str) -> String {
    let mut open = false;
    let mut word = String::new();
    let mut out = String::with_capacity(html.len() * 3);

    for c in html.chars() {
        if c == '[' && !open {
            open = true;
        } else if c == ']' && open {
            open = false;
            let link = word.to_lowercase();
            if wiki_page_names().contains(&link) {
                out.push_str(&format!(r#"<a href="{}">{}</a>"#, link, word));
                println!(r#"<a href="{}">{}</a>"#, link, word);
            } else {
                out.push('[');
                out.push_str(&word);
                out.push(']');
            }
            word.clear();
        } else if open {
            println!("WIDE OPEN");
            word.push(c);
        } else {
            println!("UM WHY");
            out.push(c);
        }
    }

    out
}

/// Convert raw Markdown into HTML.
fn markdown_to_html(md: &str) -> String {
    let mut options = markdown::Options::empty();
    options.insert(markdown::Options::ENABLE_TASKLISTS);
    options.insert(markdown::Options::ENABLE_FOOTNOTES);

    let mut wiki_link = false;
    let mut wiki_link_text = String::new();

    let parser = markdown::Parser::new_ext(&md, options).map(|event| match event {
        markdown::Event::Text(text) => {
            if text.as_ref() == "[" && !wiki_link {
                wiki_link = true;
                markdown::Event::Text("".into())
            } else if text.as_ref() == "]" && wiki_link {
                wiki_link = false;
                let page_name = wiki_link_text.to_lowercase();
                let link_text = wiki_link_text.clone();
                wiki_link_text.clear();
                if wiki_page_names().contains(&page_name) {
                    markdown::Event::Html(
                        format!(r#"<a href="/{}">{}</a>"#, page_name, link_text).into(),
                    )
                } else {
                    markdown::Event::Text(format!("[{}]", link_text).into())
                }
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

/// Path of wiki page on disk, if it exists.
/// Always in the `wiki/` directory.
/// Eg. wiki_path("Welcome") -> "wiki/welcome.md"
fn wiki_path(path: &str) -> Option<String> {
    let path = format!(
        "./wiki/{}.md",
        path.to_lowercase()
            .trim_start_matches('/')
            .replace("..", ".")
    );
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
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

/// Path of asset on disk, if it exists.
/// Always in the `web/` directory.
/// Eg web_path("style.css") -> "web/style.css"
fn web_path(path: &str) -> Option<String> {
    let path = format!("./web/{}", path.trim_start_matches('/').replace("..", "."));
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// Content type for a file on disk. We only look in `web/`.
fn get_content_type(path: &str) -> Option<&'static str> {
    let disk_path = web_path(path);
    if disk_path.is_none() {
        return None;
    }
    let disk_path = disk_path.unwrap();
    let path = Path::new(&disk_path);
    let extension = match path.extension() {
        None => return Some("text/plain"),
        Some(e) => e,
    };

    Some(match extension.to_str().unwrap() {
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
    })
}
