//! Single Wiki Page

use {
    hatter::{Object, Value},
    std::{cell::RefCell, fs, rc::Rc},
};

#[derive(Debug)]
pub struct Page {
    path: String,
    root: String,
    body: RefCell<String>,
}

/// thing.into()
impl From<Page> for Value {
    fn from(p: Page) -> Self {
        Value::Object(Rc::new(p))
    }
}

/// In Hatter: page.title, page.url, page.path
impl Object for Page {
    fn typename(&self) -> &str {
        "Page"
    }

    fn get(&self, key: &str) -> Option<Value> {
        match key {
            "title" => Some(self.title().into()),
            "name" => Some(self.name().into()),
            "url" => Some(self.url().into()),
            "path" => Some(self.path().into()),
            "body" => Some(self.body().into()),
            _ => None,
        }
    }
}

impl Page {
    pub fn new<S: AsRef<str>, T: AsRef<str>>(root: S, path: T) -> Page {
        Page {
            root: root.as_ref().into(),
            path: path.as_ref().into(),
            body: RefCell::new(String::new()),
        }
    }

    pub fn name(&self) -> &str {
        self.path_without_root().trim_end_matches(".md")
    }

    pub fn url(&self) -> String {
        format!("/{}", self.name())
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn path_without_root(&self) -> &str {
        self.path
            .trim_start_matches(&self.root)
            .trim_start_matches('.')
            .trim_start_matches('/')
    }

    pub fn body(&self) -> String {
        if self.body.borrow().is_empty() {
            self.body
                .borrow_mut()
                .push_str(&fs::read_to_string(self.path()).unwrap_or_else(|_| "".into()));
        }
        self.body.borrow().clone()
    }

    pub fn title(&self) -> String {
        self.name()
            .split('_')
            .map(|part| {
                if part.contains('/') {
                    let mut parts = part.split('/').rev();
                    let last = parts.next().unwrap_or("?");
                    format!(
                        "{}/{}",
                        parts.rev().collect::<Vec<_>>().join("/"),
                        capitalize(last)
                    )
                } else {
                    capitalize(&part)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap_or('?').to_uppercase(),
        &s.chars().skip(1).collect::<String>()
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name() {
        let page = Page::new("./wiki", "./wiki/info.md");
        assert_eq!(page.name(), "info");
        assert_eq!(page.title(), "Info");
        assert_eq!(page.url(), "/info");
        assert_eq!(page.path, "./wiki/info.md");

        let page = Page::new("./wiki", "./wiki/linux_laptops.md");
        assert_eq!(page.name(), "linux_laptops");
        assert_eq!(page.title(), "Linux Laptops");
        assert_eq!(page.url(), "/linux_laptops");
        assert_eq!(page.path, "./wiki/linux_laptops.md");
    }
}
