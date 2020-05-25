//! (Method, URL) => Code

use {
    crate::{helper::*, render, util},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        fs,
        io::{self, Write},
        path::Path,
    },
};

#[macro_use]
use vial::prelude::*;

vial! {
    GET "/" => index;

    GET "/sleep" => |_| {
        std::thread::sleep(std::time::Duration::from_secs(5));
        "Zzzzz..."
    };

    GET "/new" => new;
    POST "/new" => create;

    GET "/:page" => show;
    POST "/:page" => update;
}

fn new2(req: Request) -> impl Responder {
    html! {
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
                    value=name,
                    id="focused"
                );
            }
            textarea(name="markdown", id="markdown") {
                : format!("# {}", name);
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

fn index(req: Request) -> Result<impl Responder, io::Error> {
    render::layout(
        "deadwiki",
        &format!(
            "{}",
            asset::to_string("index.html")?.replace(
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

fn update(req: Request) -> impl Responder {
    if query.is_empty() {
        status = 404;
        body = asset::to_string("404.html")?;
    } else {
        if let Some(disk_path) = render::page_path(path) {
            let mut content = String::new();
            req.as_reader().read_to_string(&mut content)?;
            let mdown = content.split("markdown=").last().unwrap_or("");
            let af = AtomicFile::new(disk_path, AllowOverwrite);
            af.write(|f| f.write_all(util::decode_form_value(mdown).as_bytes()))?;
            status = 302;
            body = path.to_string();
        } else {
            status = 404;
            body = asset::to_string("404.html")?;
        }
    }
}

fn show(req: Request) -> impl Responder {
    if let Some(disk_path) = render::page_path(path) {
        status = 200;
        if query.is_empty() {
            body = render::page(req, path).unwrap_or_else(|_| "".into());
        } else if query == "edit" {
            body = render::layout(
                "Edit",
                &asset::to_string("edit.html")?
                    .replace("{markdown}", &fs::read_to_string(disk_path)?),
                None,
            )
        }
    } else if asset::exists(path) {
        status = 200;
        body = asset::to_string(path)?;
        content_type = util::get_content_type(path);
    } else {
        status = 404;
        body = asset::to_string("404.html")?;
    }
}
