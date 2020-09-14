//! Convert wiki Markdown to HTML.
//!
//! Supports two syntax extensions:
//!
//! - [Page] to link directly to a wiki page.
//! - #tag to link to a hashtag.
//!

use pulldown_cmark as markdown;

/// Convert raw wiki Markdown into HTML.
/// Takes a list of all wiki pages in the system, for [Link]s.
pub fn to_html(md: &str, names: &[String]) -> String {
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
                let page_exists = names.contains(&page_name);
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
