//! (Method, URL) => Code

use {
    crate::{helper::*, render, util, wiki_root},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        collections::HashMap,
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

// Don't include the '#' when you search, eg pass in "hashtag" to
// search for #hashtag.
fn pages_with_tag(tag: &str) -> Result<Vec<String>, io::Error> {
    let tag = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };

    let out = shell!("grep --exclude-dir .git -l -r '{}' {}", tag, wiki_root())?;
    Ok(out
        .split("\n")
        .filter_map(|line| {
            if !line.is_empty() {
                Some(
                    line.split(':')
                        .next()
                        .unwrap_or("?")
                        .trim_end_matches(".md")
                        .trim_start_matches(&wiki_root())
                        .trim_start_matches('/')
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

fn search(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(tag) = req.query("tag") {
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
pub fn index(_req: Request) -> Result<impl Responder, io::Error> {
    let mut folded = "";

    Ok(render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("html/index.html")?
                .replace(
                    "{empty-list-msg}",
                    if page_names().is_empty() {
                        "<i>Wiki Pages you create will show up here.</i>"
                    } else {
                        ""
                    }
                )
                .replace(
                    "{pages}",
                    &page_names()
                        .iter()
                        .map(|name| {
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
                                name,
                                wiki_path_to_title(name)
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
    if !page_names().contains(&path) {
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
fn recent(_: Request) -> Result<impl Responder, io::Error> {
    let out = shell!(
        r#"git --git-dir={}/.git log --pretty=format: --name-only -n 30 | grep "\.md\$""#,
        wiki_root()
    )?;
    let mut pages = vec![];
    let mut seen = HashMap::new();
    for page in out.split("\n") {
        if seen.get(page).is_some() || page == ".md" || page.is_empty() {
            // TODO: .md hack
            continue;
        } else {
            pages.push(page);
            seen.insert(page, true);
        }
    }

    render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("html/list.html")?
                .replace(
                    "{empty-list-msg}",
                    if pages.is_empty() {
                        "<i>No Wiki Pages found.</i>"
                    } else {
                        ""
                    }
                )
                .replace(
                    "{pages}",
                    &pages
                        .iter()
                        .map(|page| {
                            format!(
                                "<li><a href='/{}'>{}</a></li>",
                                page.replace(".md", ""),
                                &wiki_path_to_title(page)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("")
                )
        ),
        Some(&nav("/")?),
    )
}

fn jump(_: Request) -> Result<impl Responder, io::Error> {
    let partial = asset::to_string("html/_jump_page.html")?;
    if page_names().is_empty() {
        return Ok("Add a few wiki pages then come back.".to_string());
    }

    let mut id = -1;
    let mut entries = page_names()
        .iter()
        .map(|page| {
            id += 1;
            partial
                .replace("{page.id}", &format!("{}", id))
                .replace("{page.path}", page)
                .replace("{page.name}", &wiki_path_to_title(page))
        })
        .collect::<Vec<_>>();
    entries.extend(tag_names().iter().map(|tag| {
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
        if let Some(_disk_path) = page_path(name) {
            return Ok(render::page(name)?.to_response());
        }
    }
    Ok(response_404())
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.html")
}
