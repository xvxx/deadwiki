use {
    crate::{db::ReqWithDB, markdown, Hatter},
    hatter,
    std::{collections::HashMap, io, time::Instant},
    vial::prelude::*,
};

routes! {
    GET "/" => index;
    GET "/all" => all_pages;

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
    let mut env = Hatter::new();
    let tag = unwrap_or_404!(req.query("tag"));
    env.set("tag", tag);
    env.set("pages", req.db().find_pages_with_tag(tag)?);
    render("Search", env.render("html/search.hat")?)
}

fn new(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();
    env.set("name", req.query("name"));
    render("New Page", env.render("html/new.hat")?)
}

/// Render the index page which lists all wiki pages or displays your
/// `index.md` wiki page.
fn index(req: Request) -> io::Result<impl Responder> {
    if req.db().exists("index") {
        show_page(&req, "index")
    } else {
        show_index(&req)
    }
}

/// List all wiki pages.
fn all_pages(req: Request) -> io::Result<impl Responder> {
    show_index(&req)
}

// POST new page
fn create(req: Request) -> io::Result<impl Responder> {
    let name = req.form("name").unwrap_or("note.md");
    let page = req.db().create(name, req.form("markdown").unwrap_or(""))?;
    redirect_to(page.url())
}

// Recently modified wiki pages.
fn recent(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();
    env.set("pages", req.db().recent()?);
    render("Recently Modified Pages", env.render("html/recent.hat")?)
}

fn jump(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();

    let pages = req.db().pages()?;
    let pages = pages.iter().enumerate().map(|(i, p)| {
        let mut map: HashMap<&str, hatter::Value> = HashMap::new();
        map.insert("id", i.into());
        map.insert("name", p.title().into());
        map.insert("url", p.url().into());
        map
    });

    let idx = pages.len();
    let tags = req.db().tags()?;
    let tags = tags.iter().enumerate().map(|(i, tag)| {
        let mut map: HashMap<&str, hatter::Value> = HashMap::new();
        map.insert("id", (idx + i).into());
        map.insert("name", format!("#{}", tag).into());
        map.insert("url", format!("/search?tag={}", tag).into());
        map
    });

    env.set("pages", pages.chain(tags).collect::<Vec<_>>());
    render("Jump to Wiki Page", env.render("html/jump.hat")?)
}

fn update(req: Request) -> io::Result<impl Responder> {
    let name = unwrap_or_404!(req.arg("name"));
    let page = req.db().update(name, &markdown_post_data(&req))?;
    redirect_to(page.url())
}

fn edit(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();
    let name = unwrap_or_404!(req.arg("name"));
    let page = unwrap_or_404!(req.db().find(name));
    env.set("page", page);
    env.set("conflicts", req.query("conflicts").is_some());
    render("Edit", env.render("html/edit.hat")?)
}

fn show(req: Request) -> io::Result<impl Responder> {
    let name = unwrap_or_404!(req.arg("name"));
    if name.ends_with(".md") || !name.contains('.') {
        show_page(&req, name)
    } else {
        Ok(Response::from_file(&req.db().absolute_path(name)))
    }
}

fn show_index(req: &Request) -> io::Result<Response> {
    let mut env = Hatter::new();
    env.set("pages", req.db().pages()?);
    env.set("nested_header", |args: hatter::Args| {
        Ok(args.need_string(0)?.split('/').next().unwrap_or("").into())
    });
    env.set("nested_title", |args: hatter::Args| {
        Ok(args
            .need_string(0)?
            .split('/')
            .skip(1)
            .collect::<Vec<_>>()
            .join("/")
            .into())
    });
    env.set("nested?", |args: hatter::Args| {
        Ok(args.need_string(0)?.contains('/').into())
    });
    render("deadwiki", env.render("html/index.hat")?)
}

fn show_page(req: &Request, name: &str) -> io::Result<Response> {
    let mut env = Hatter::new();
    let page = unwrap_or_404!(req.db().find(name.trim_end_matches(".md")));
    if page.has_conflict() {
        return redirect_to(format!("/edit{}?conflicts=true", page.url()));
    }
    let title = page.title().clone();
    let names = req.db().names()?;
    env.set("page", page);
    env.set("markdown", move |args: hatter::Args| {
        let src = args.need_string(0).unwrap();
        Ok(markdown::to_html(&src, &names).into())
    });
    render(&title, env.render("html/show.hat")?)
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.html")
}

// Clean up POST'd markdown data - mostly by removing \r, which HTTP loves.
fn markdown_post_data(req: &Request) -> String {
    req.form("markdown").unwrap_or("").replace('\r', "")
}

fn redirect_to<S: AsRef<str>>(path: S) -> io::Result<Response> {
    Ok(Response::redirect_to(path.as_ref()))
}

fn render<S: AsRef<str>>(title: &str, body: S) -> Result<Response, io::Error> {
    let mut env = Hatter::new();
    env.set("title", title);
    env.set("body", body.as_ref());
    let start = Instant::now();
    let html = env.render("html/layout.hat")?;
    let end = start.elapsed();
    Ok(Response::from(
        html.replace("$render-time", &format!(r#""{:?}""#, end)),
    ))
}
