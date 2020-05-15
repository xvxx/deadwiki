//! The code that gets run when a page is visited.
use {
    crate::{asset, render, util, Request},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        fs,
        io::{self, Write},
        path::Path,
    },
    tiny_http::Method::{Get, Post},
};

/// Route a Request to (Status Code, Body, Content-Type)
pub fn route(req: &mut Request) -> Result<(i32, String, &'static str), io::Error> {
    let mut status = 404;
    let mut body = "404 Not Found".to_string();
    let mut content_type = "text/html; charset=utf8";

    let full_url = req.url().to_string();
    let mut parts = full_url.splitn(2, "?");
    let (url, query) = (parts.next().unwrap_or("/"), parts.next().unwrap_or(""));

    match (req.method(), url) {
        (Get, "/") => {
            status = 200;
            body = render::index(&req)?;
        }
        (Get, "/new") => {
            status = 200;
            let name = req.param("name");

            body = render::layout(
                "new page",
                &asset::to_string("new.html")?.replace("{name}", &name),
                None,
            );
        }
        (Get, "/sleep") => {
            status = 200;
            body = "Zzzzz...".into();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        (Get, "/404") => {
            status = 404;
            body = asset::to_string("404.html")?;
        }
        (Post, "/new") => {
            let path = render::pathify(&req.param("name"));
            if !req.page_names().contains(&path) {
                if let Some(disk_path) = render::new_page_path(&path) {
                    if disk_path.contains('/') {
                        if let Some(dir) = Path::new(&disk_path).parent() {
                            fs::create_dir_all(&dir.display().to_string())?;
                        }
                    }
                    let mut file = fs::File::create(disk_path)?;
                    let mdown = req.param("markdown");
                    write!(file, "{}", mdown)?;
                    status = 302;
                    body = path.to_string();
                }
            }
        }
        (Post, path) => {
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

        (Get, path) => {
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

        (x, y) => println!("x: {:?}, y: {:?}", x, y),
    }
    Ok((status, body, content_type))
}
