//! Rendering "logic".

use {
    crate::{asset, render, util::shell, wiki_root, Request},
    pulldown_cmark as markdown,
    std::{fs, io, os::unix::fs::PermissionsExt, path::Path, str},
};

/// Render the index page which lists all wiki pages.
pub fn index(req: &Request) -> Result<String, io::Error> {
    let mut folded = "";

    Ok(render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("index.html")?
                .replace(
                    "{empty-list-msg}",
                    if req.page_names().is_empty() {
                        "<i>Wiki Pages you create will show up here.</i>"
                    } else {
                        ""
                    }
                )
                .replace(
                    "{pages}",
                    &req.page_names()
                        .iter()
                        .map(|name| {
                            let mut prefix = "".to_string();
                            if let Some(idx) = name.find('/') {
                                if folded.is_empty() {
                                    folded = &name[..=idx];
                                    prefix = format!("<details><summary>{}</summary>", folded);
                                } else if folded != &name[..=idx] {
                                    folded = &name[..=idx];
                                    prefix =
                                        format!("</details><details><summary>{}</summary>", folded);
                                }
                            } else if !folded.is_empty() {
                                prefix = "</details>".to_string();
                                folded = "";
                            }

                            format!(
                                "{}  <li><a href='{}'>{}</a></li>\n",
                                prefix,
                                name,
                                wiki_path_to_title(name)
                            )
                        })
                        .collect::<String>()
                )
        ),
        &req.page_names(),
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
            render::layout(
                &title,
                &markdown_to_html(&req, &html),
                &req.page_names(),
                Some(&nav()?),
            )
        })
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", path),
        ))
    }
}

// #hashtag results
pub fn search(req: &mut Request) -> Result<String, io::Error> {
    let tag = &req.param("tag");
    Ok(render::layout(
        "search",
        &asset::to_string("html/search.html")?
            .replace("{tag}", &format!("#{}", tag))
            .replace(
                "{results}",
                &pages_with_tag(tag)?
                    .iter()
                    .map(|page| {
                        format!(
                            "<li><a href='/{}'>{}</a></li>",
                            page,
                            wiki_path_to_title(page)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        &req.page_names(),
        Some("<a href='/'>home</a>"),
    ))
}

/// Renders a chunk of HTML surrounded by `static/layout.html`.
pub fn layout(title: &str, body: &str, pages: &[String], nav: Option<&str>) -> String {
    let mut webview_app = "";
    if cfg!(feature = "gui") {
        webview_app = "webview-app";
    }

    if asset::exists("layout.html") {
        asset::to_string("layout.html")
            .unwrap_or_else(|_| "".into())
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{webview-app}", webview_app)
            .replace("{pages.json}", &pages_as_json(pages))
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    }
}

/// Convert raw Markdown into HTML.
fn markdown_to_html(req: &Request, md: &str) -> String {
    let mut options = markdown::Options::empty();
    options.insert(markdown::Options::ENABLE_TABLES);
    options.insert(markdown::Options::ENABLE_FOOTNOTES);
    options.insert(markdown::Options::ENABLE_STRIKETHROUGH);
    options.insert(markdown::Options::ENABLE_TASKLISTS);

    // are we parsing a wiki link like [Help] or [Solar Power]?
    let mut wiki_link = false;
    // if we are, store the text between [ and ]
    let mut wiki_link_text = String::new();

    let parser = markdown::Parser::new_ext(&md, options).map(|event| match event {
        markdown::Event::Text(text) => {
            let text = text.replace("<", "&lt;").replace(">", "&gt;");

            if &text == "[" && !wiki_link {
                wiki_link = true;
                markdown::Event::Text("".into())
            } else if &text == "]" && wiki_link {
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
                if text.contains("http://") || text.contains("https://") {
                    let linked = autolink::auto_link(&text, &[]);
                    if linked.len() == text.len() {
                        markdown::Event::Text(text.into())
                    } else {
                        markdown::Event::Html(linked.into())
                    }
                } else if let Some(idx) = text.find('#') {
                    // look for and link #hashtags
                    let linked = text[idx..]
                        .split(' ')
                        .map(|word| {
                            if word.starts_with('#')
                                && word.len() > 1
                                && word.chars().nth(1).unwrap_or('?').is_alphanumeric()
                            {
                                let word = word.trim_start_matches('#');
                                format!("<a href='/search?tag={}'>#{}</a>", word, word)
                            } else {
                                word.into()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    markdown::Event::Html(format!("{}{}", &text[..idx], linked).into())
                } else {
                    markdown::Event::Text(text.into())
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
        .trim_end_matches(".md")
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
    format!("{}.md", pathify(path))
}

// Don't include the '#' when you search, eg pass in "hashtag" to
// search for #hashtag.
fn pages_with_tag(tag: &str) -> Result<Vec<String>, io::Error> {
    let tag = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };

    let out = shell("grep", &["-r", &tag, "."])?;
    let mut pages = out
        .split("\n")
        .filter_map(|line| {
            if !line.is_empty() {
                Some(
                    line.split(':')
                        .next()
                        .unwrap_or("?")
                        .trim_end_matches(".md")
                        .trim_start_matches(&wiki_root())
                        .trim_start_matches('.')
                        .trim_start_matches('/')
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    pages.sort();
    pages.dedup();
    Ok(pages)
}

/// [{ title: "My Page", path: "my_page" }]
fn pages_as_json(page_names: &[String]) -> String {
    format!(
        "[{}]",
        page_names
            .iter()
            .map(|page| {
                format!(
                    r#"{{ name: "{}", path: "{}" }}"#,
                    wiki_path_to_title(page),
                    page
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    )
}
