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

    GET "/toggle-ui-mode" => toggle_ui_mode;

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
    req.render("Search", env.render("html/search.hat")?)
}

fn new(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();
    env.set("error?", false);
    env.set("name", req.query("name"));
    env.set(
        "page-body",
        format!("# {}", req.query("name").unwrap_or("")),
    );
    req.render("New Page", env.render("html/new.hat")?)
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

/// GET /toggle-ui-mode
fn toggle_ui_mode(req: Request) -> impl Responder {
    let mut res = Response::redirect_to("/");
    if matches!(req.cookie("ui-mode"), Some("dark")) {
        res.set_cookie("ui-mode", "light");
    } else {
        res.set_cookie("ui-mode", "dark");
    }
    res
}

// POST new page
fn create(req: Request) -> io::Result<impl Responder> {
    let name = req.form("name").unwrap_or("note.md");
    if !req.db().exists(name) {
        let page = req.db().create(name, req.form("markdown").unwrap_or(""))?;
        redirect_to(page.url())
    } else {
        let mut env = Hatter::new();
        env.set("name", name);
        env.set("error?", true);
        env.set("error", "Wiki page with that name already exists.");
        env.set("page-body", req.form("markdown").unwrap_or(""));
        req.render("New Page", env.render("html/new.hat")?)
    }
}

// Recently modified wiki pages.
fn recent(req: Request) -> io::Result<impl Responder> {
    let mut env = Hatter::new();
    env.set("is_git?", req.db().is_git());
    env.set("pages", req.db().recent()?);
    req.render("Recently Modified Pages", env.render("html/recent.hat")?)
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
    req.render("Jump to Wiki Page", env.render("html/jump.hat")?)
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
    req.render("Edit", env.render("html/edit.hat")?)
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
    req.render("deadwiki", env.render("html/index.hat")?)
}

fn show_page(req: &Request, name: &str) -> io::Result<Response> {
    let mut env = Hatter::new();
    let page = unwrap_or_404!(req.db().find(name.trim_end_matches(".md")));
    if page.has_conflict() {
        return redirect_to(format!("/edit{}?conflicts=true", page.url()));
    }

    let path = req.path().trim_start_matches('/');
    env.set(
        "new-link",
        if path.contains('/') {
            format!(
                "/new?name={}/",
                path.split('/')
                    .take(path.matches('/').count())
                    .collect::<Vec<_>>()
                    .join("/")
            )
        } else {
            "/new".into()
        },
    );

    let title = page.title().clone();
    let names = req.db().names()?;

    env.set("page", page);
    env.set("markdown", move |args: hatter::Args| {
        let src = args.need_string(0).unwrap();
        Ok(markdown::to_html(&src, &names).into())
    });
    req.render(&title, env.render("html/show.hat")?)
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

trait Render {
    fn render<S: AsRef<str>>(&self, title: &str, body: S) -> Result<Response, io::Error>;
}

impl Render for Request {
    fn render<S: AsRef<str>>(&self, title: &str, body: S) -> Result<Response, io::Error> {
        let mut env = Hatter::new();
        env.set("title", title);
        env.set("body", body.as_ref());
        env.set("dark-mode?", matches!(self.cookie("ui-mode"), Some("dark")));
        let start = Instant::now();
        let html = env.render("html/layout.hat")?;
        let end = start.elapsed();
        Ok(Response::from(
            html.replace("$render-time", &format!(r#""{:?}""#, end)),
        ))
    }
}
