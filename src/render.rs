//! Rendering "logic".

use {
    crate::{helper::*, render, util::shell},
    pulldown_cmark as markdown,
    std::{fs, io, str},
    vial::asset,
};

/// Render a wiki page to a fully loaded HTML string, with layout.
pub fn page(path: &str) -> Result<String, io::Error> {
    let raw = path.ends_with(".md");
    let orig_path = path;
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
            fs::read_to_string(&path).unwrap_or_else(|_| "".into())
        };
        if raw {
            Ok(format!("<pre>{}</pre>", html))
        } else {
            render::layout(&title, &markdown_to_html(&html), Some(&nav(&orig_path)?))
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} not found", path),
        ))
    }
}

/// Renders a chunk of HTML surrounded by `static/html/layout.html`.
pub fn layout<T, S>(title: T, body: S, nav: Option<&str>) -> Result<String, io::Error>
where
    T: AsRef<str>,
    S: AsRef<str>,
{
    let title = title.as_ref();
    let body = body.as_ref();
    let mut webview_app = "";
    if cfg!(feature = "gui") {
        webview_app = "webview-app";
    }

    Ok(if asset::exists("html/layout.html") {
        asset::to_string("html/layout.html")?
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{webview-app}", webview_app)
            .replace("{pages.json}", &pages_as_json())
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    })
}

/// Convert raw Markdown into HTML.
fn markdown_to_html(md: &str) -> String {
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
            if *text == *"[" && !wiki_link {
                wiki_link = true;
                markdown::Event::Text("".into())
            } else if *text == *"]" && wiki_link {
                wiki_link = false;
                let page_name = wiki_link_text.to_lowercase().replace(" ", "_");
                let link_text = wiki_link_text.clone();
                wiki_link_text.clear();
                let page_exists = page_names().contains(&page_name);
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
                            if word.starts_with('#') && word.len() > 1 {
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
