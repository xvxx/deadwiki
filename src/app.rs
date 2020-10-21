//! (Method, URL) => Code

use {
    crate::{db::ReqWithDB, helper::*, render},
    std::{
        fs,
        io::{self},
    },
    vial::prelude::*,
};

routes! {
    GET "/" => index;

    GET "/jump" => jump;
    GET "/recent" => recent;

    GET "/new" => new;
    POST "/new" => create;

    GET "/search" => search;

    GET "/edit/*name" => edit;
    POST "/edit/*name" => update;
    GET "/*name" => show;
}

/// Helper for checking for presence of form/query params.
macro_rules! unwrap_or_404 {
    ($expr:expr) => {
        if let Some(v) = $expr {
            v
        } else {
            return Ok(response_404());
        }
    };
}

fn search(req: Request) -> io::Result<impl Responder> {
    let tag = unwrap_or_404!(req.query("tag"));
    Ok(render::layout(
        "search",
        &asset::to_string("html/search.html")?
            .replace("{tag}", &format!("#{}", tag))
            .replace(
                "{results}",
                &req.db()
                    .find_pages_with_tag(tag)?
                    .iter()
                    .map(|page| format!("<li><a href='{}'>{}</a></li>", page.url(), page.title()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        None,
    )?
    .to_response())
}

fn new(req: Request) -> io::Result<impl Responder> {
    render::layout(
        "new page",
        &asset::to_string("html/new.html")?.replace("{name}", &req.query("name").unwrap_or("")),
        None,
    )
}

/// Render the index page which lists all wiki pages.
fn index(req: Request) -> io::Result<impl Responder> {
    let mut folded = "";

    Ok(render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("html/index.html")?
                .replace(
                    "{empty-list-msg}",
                    if req.db().is_empty() {
                        "<i>Wiki Pages you create will show up here.</i>"
                    } else {
                        ""
                    }
                )
                .replace(
                    "{pages}",
                    &req.db()
                        .pages()?
                        .iter()
                        .map(|page| {
                            let name = page.name();
                            let mut prefix = "".to_string();
                            if let Some(idx) = name.trim_start_matches('/').find('/') {
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
                                page.url(),
                                page.title(),
                            )
                        })
                        .collect::<String>()
                )
        ),
        None,
    ))
}

// POST new page
fn create(req: Request) -> io::Result<impl Responder> {
    let name = req.form("name").unwrap_or("note.md");
    let page = req.db().create(name, req.form("markdown").unwrap_or(""))?;
    Ok(Response::redirect_to(page.url()))
}

// Recently modified wiki pages.
fn recent(req: Request) -> io::Result<impl Responder> {
    render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("html/list.html")?
                .replace(
                    "{empty-list-msg}",
                    if req.db().is_empty() {
                        "<i>No Wiki Pages found.</i>"
                    } else {
                        ""
                    }
                )
                .replace(
                    "{pages}",
                    &req.db()
                        .recent()?
                        .iter()
                        .map(|page| {
                            format!("<li><a href='{}'>{}</a></li>", page.url(), page.title())
                        })
                        .collect::<Vec<_>>()
                        .join("")
                )
        ),
        Some(&nav("/")?),
    )
}

fn jump(req: Request) -> io::Result<impl Responder> {
    let partial = asset::to_string("html/_jump_page.html")?;
    if req.db().is_empty() {
        return Ok("Add a few wiki pages then come back.".to_string());
    }

    let mut id = -1;
    let mut entries = req
        .db()
        .pages()?
        .iter()
        .map(|page| {
            id += 1;
            partial
                .replace("{page.id}", &format!("{}", id))
                .replace("{page.path}", &page.url())
                .replace("{page.name}", &page.title())
        })
        .collect::<Vec<_>>();
    entries.extend(req.db().tags()?.iter().map(|tag| {
        id += 1;
        partial
            .replace("{page.id}", &format!("{}", id))
            .replace("{page.path}", &format!("search?tag={}", tag))
            .replace("{page.name}", &format!("#{}", tag))
    }));

    render::layout(
        "Jump to Wiki Page",
        asset::to_string("html/jump.html")?.replace("{pages}", &format!("{}", entries.join(""))),
        None,
    )
}

fn update(req: Request) -> io::Result<impl Responder> {
    if let Some(name) = req.arg("name") {
        let page = req.db().update(name, req.form("markdown").unwrap_or(""))?;
        Ok(Response::redirect_to(page.url()))
    } else {
        Ok(Response::from(404))
    }
}

fn edit(req: Request) -> io::Result<impl Responder> {
    let name = unwrap_or_404!(req.arg("name"));
    let page = unwrap_or_404!(req.db().find(name));

    Ok(render::layout(
        "Edit",
        &asset::to_string("html/edit.html")?
            .replace("{markdown}", &fs::read_to_string(page.path())?),
        None,
    )?
    .to_response())
}

fn show(req: Request) -> io::Result<impl Responder> {
    let name = unwrap_or_404!(req.arg("name"));
    let raw = name.ends_with(".md");
    let page = unwrap_or_404!(req.db().find(name.trim_end_matches(".md")));
    Ok(render::page(page, raw, &req.db().names()?)?.to_response())
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.html")
}
