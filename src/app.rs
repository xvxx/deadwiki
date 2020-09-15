//! (Method, URL) => Code

use {
    crate::{db::ReqWithDB, helper::*, render},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        fs,
        io::{self, Write},
        path::Path,
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

fn search(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(tag) = req.query("tag") {
        Ok(render::layout(
            "search",
            &asset::to_string("html/search.html")?
                .replace("{tag}", &format!("#{}", tag))
                .replace(
                    "{results}",
                    &req.db()
                        .find_pages_with_tag(tag)?
                        .iter()
                        .map(|page| {
                            format!("<li><a href='{}'>{}</a></li>", page.url(), page.title())
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                ),
            None,
        )?
        .to_response())
    } else {
        Ok(Response::from(404))
    }
}

fn new(req: Request) -> Result<impl Responder, io::Error> {
    render::layout(
        "new page",
        &asset::to_string("html/new.html")?.replace("{name}", &req.query("name").unwrap_or("")),
        None,
    )
}

/// Render the index page which lists all wiki pages.
fn index(req: Request) -> Result<impl Responder, io::Error> {
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

fn create(req: Request) -> Result<impl Responder, io::Error> {
    let path = pathify(&req.form("name").unwrap_or(""));
    if !req.db().names()?.contains(&path) {
        if let Some(disk_path) = new_page_path(&path) {
            if disk_path.contains('/') {
                if let Some(dir) = Path::new(&disk_path).parent() {
                    fs::create_dir_all(&dir.display().to_string())?;
                }
            }
            let mut file = fs::File::create(disk_path)?;
            return if let Some(mdown) = req.form("markdown") {
                write!(file, "{}", mdown)?;
                Ok(Response::redirect_to(path))
            } else {
                Ok(Response::redirect_to("/new"))
            };
        }
    }
    Ok(response_404())
}

// Recently modified wiki pages.
fn recent(req: Request) -> Result<impl Responder, io::Error> {
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

fn jump(req: Request) -> Result<impl Responder, io::Error> {
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

fn update(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        if let Some(disk_path) = page_path(name) {
            let mdown = req.form("markdown").unwrap_or("");
            let af = AtomicFile::new(disk_path, AllowOverwrite);
            af.write(|f| f.write_all(mdown.as_bytes()))?;
            return Ok(Response::redirect_to(format!(
                "/{}",
                pathify(name).replace("edit/", "")
            )));
        }
    }
    Ok(response_404())
}

fn edit(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        if let Some(disk_path) = page_path(name) {
            return Ok(render::layout(
                "Edit",
                &asset::to_string("html/edit.html")?
                    .replace("{markdown}", &fs::read_to_string(disk_path)?),
                None,
            )?
            .to_response());
        }
    }
    Ok(response_404())
}

fn show(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        let raw = name.ends_with(".md");
        if let Some(page) = req.db().find(name.trim_end_matches(".md")) {
            return Ok(render::page(page, raw, &req.db().names()?)?.to_response());
        }
    }
    Ok(response_404())
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.html")
}
