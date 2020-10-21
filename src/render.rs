//! Rendering "logic".

use {
    crate::{helper::*, markdown, render, Page},
    std::{fs, io, str},
    vial::asset,
};

/// Render a wiki page to a fully loaded HTML string, with layout.
pub fn page(page: Page, raw: bool, page_names: &[String]) -> io::Result<String> {
    let html = if is_executable(&page.path()) {
        shell!(page.path()).unwrap_or_else(|e| e.to_string())
    } else {
        fs::read_to_string(&page.path()).unwrap_or_else(|_| "".into())
    };
    if raw {
        Ok(format!("<pre>{}</pre>", html))
    } else {
        render::layout(
            &page.title(),
            &markdown::to_html(&html, page_names),
            Some(&nav(page.name())?),
        )
    }
}

/// Renders a chunk of HTML surrounded by `static/html/layout.html`.
pub fn layout<T, S>(title: T, body: S, nav: Option<&str>) -> io::Result<String>
where
    T: AsRef<str>,
    S: AsRef<str>,
{
    let title = title.as_ref();
    let body = body.as_ref();

    Ok(if asset::exists("html/layout.html") {
        asset::to_string("html/layout.html")?
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    })
}
