//! (Method, URL) => Code

use {
    crate::{helper::*, render},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        fs,
        io::{self, Write},
        path::Path,
    },
};

use vial::owned_html;
use vial::prelude::*;

vial! {
    GET "/" => index;

    GET "/sleep" => |_| {
        std::thread::sleep(std::time::Duration::from_secs(5));
        "Zzzzz..."
    };

    GET "/new" => new;
    POST "/new" => create;

    GET "/:name" => show;
    GET "/:name/edit" => edit;
    POST "/:name" => update;
}

#[allow(dead_code)]
fn new2(req: Request) -> impl Responder {
    owned_html! {
        p {
            a(href="/") { : "home" }
            a(href="javascript:history.back()") { : "back" }
        }

        form(method="POST", action="/new", id="form") {
            p {
                input(
                    name="name",
                    type="text",
                    placeholder="filename",
                    value=req.query("name").unwrap_or(""),
                    id="focused"
                );
            }
            textarea(name="markdown", id="markdown") {
                : format!("# {}", req.query("name").unwrap_or(""));
            }
            input(type="submit");
        }
    }
}

fn new(req: Request) -> Result<impl Responder, io::Error> {
    Ok(render::layout(
        "new page",
        &asset::to_string("html/new.html")?.replace("{name}", &req.query("name").unwrap_or("")),
        None,
    ))
}

fn index(_req: Request) -> Result<impl Responder, io::Error> {
    render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("html/index.html")?.replace(
                "{pages}",
                &page_names()
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
    )
}

fn create(req: Request) -> Result<impl Responder, io::Error> {
    let path = pathify(&req.query("name").unwrap_or(""));
    if !page_names().contains(&path) {
        if let Some(disk_path) = new_page_path(&path) {
            if disk_path.contains('/') {
                if let Some(dir) = Path::new(&disk_path).parent() {
                    fs::create_dir_all(&dir.display().to_string())?;
                }
            }
            let mut file = fs::File::create(disk_path)?;
            let mdown = req.arg("markdown").unwrap_or("");
            write!(file, "{}", mdown)?;
            return Ok(Response::from(302).with_body(&path));
        }
    }
    Ok(Response::from(404))
}

fn update(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        if let Some(disk_path) = page_path(name) {
            let mdown = req.form("markdown").unwrap_or("");
            let af = AtomicFile::new(disk_path, AllowOverwrite);
            af.write(|f| f.write_all(mdown.as_bytes()))?;
            return Ok(Response::from(302).with_body(&pathify(name)));
        }
    }
    Ok(Response::from(404))
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
    Ok(Response::from(404))
}

fn show(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        return Ok(render::page(name)
            .unwrap_or_else(|_| "".into())
            .to_response());
    }
    Ok(Response::from(404).with_body("404 Not Found"))
}
