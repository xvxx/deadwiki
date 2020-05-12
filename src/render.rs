//! Rendering "logic".

use {
    crate::{asset, render, web, Request},
    pulldown_cmark as markdown,
    std::{fs, io, os::unix::fs::PermissionsExt, path::Path, process::Command, str},
};

/// Render the index page which lists all wiki pages.
pub fn index(req: &Request) -> Result<String, io::Error> {
    Ok(render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("index.html")?.replace(
                "{pages}",
                &req.page_names()
                    .iter()
                    .map(|name| format!(
                        "  <li><a href='{}'>{}</a></li>\n",
                        name,
                        wiki_path_to_title(name)
                    ))
                    .collect::<String>()
            )
        ),
        None,
    ))
}

/// Render a wiki page to a fully loaded HTML string, with layout.
pub fn page(req: &Request, path: &str) -> Result<String, io::Error> {
    let raw = path.ends_with(".md");
    let path = if raw {
        path.trim_end_matches(".md")
    } else {
        path
    };
    let title = wiki_path_to_title(path);
    if let Some(path) = page_path(path) {
        let html = if is_executable(&path) {
            shell(&path, &[]).unwrap_or_else(|e| e.to_string())
        } else {
            fs::read_to_string(path).unwrap_or_else(|_| "".into())
        };
        Ok(if raw {
            format!("<pre>{}</pre>", html)
        } else {
            render::layout(&title, &markdown_to_html(&req, &html), Some(&nav()?))
        })
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", path),
        ))
    }
}

/// Renders a chunk of HTML surrounded by `static/layout.html`.
pub fn layout(title: &str, body: &str, nav: Option<&str>) -> String {
    if asset::exists("layout.html") {
        asset::to_string("layout.html")
            .unwrap_or_else(|_| "".into())
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    }
}

/// Convert raw Markdown into HTML.
fn markdown_to_html(req: &Request, md: &str) -> String {
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
                let page_exists = req.page_names().contains(&page_name);
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

/// Return the <nav> for a page
fn nav() -> Result<String, io::Error> {
    asset::to_string("nav.html")
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

/// Convert a wiki page name or file path to cleaned up path.
/// Ex: "Test Results" -> "test_results"
pub fn pathify(path: &str) -> String {
    let path = if path.ends_with(".html") && !path.starts_with("html/") {
        format!("html/{}", path)
    } else {
        path.to_string()
    };
    path.to_lowercase()
        .trim_start_matches('/')
        .replace("..", ".")
        .replace(" ", "_")
        .chars()
        .filter(|&c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/')
        .collect::<String>()
}

/// Path of wiki page on disk, if it exists.
/// Ex: page_path("Welcome") -> "wiki/welcome.md"
pub fn page_path(path: &str) -> Option<String> {
    let path = page_disk_path(path);
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// Returns a path on disk to a new wiki page.
/// Nothing if the page already exists.
pub fn new_page_path(path: &str) -> Option<String> {
    if page_path(path).is_none() {
        Some(page_disk_path(path))
    } else {
        None
    }
}

/// Returns a wiki path on disk, regardless of whether it exists.
fn page_disk_path(path: &str) -> String {
    format!("{}/{}.md", wiki_root(), pathify(path))
}

/// Path to our deadwiki.
pub fn wiki_root() -> String {
    web::WIKI_ROOT.lock().unwrap().clone()
}
