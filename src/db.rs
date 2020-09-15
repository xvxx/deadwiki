use {
    crate::Page,
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        collections::HashMap,
        fs,
        io::{self, Write},
        path::Path,
    },
};

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub trait ReqWithDB {
    fn db(&self) -> &DB;
}

impl ReqWithDB for vial::Request {
    fn db(&self) -> &DB {
        self.state::<DB>()
    }
}

pub struct DB {
    root: String,
}

unsafe impl Sync for DB {}
unsafe impl Send for DB {}

impl DB {
    /// Create a new DB object. Should only have one per run.
    pub fn new<S: AsRef<str>>(root: S) -> DB {
        DB {
            root: root.as_ref().to_string(),
        }
    }

    /// Is this DB empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many wiki pages have been created?
    pub fn len(&self) -> usize {
        if let Ok(res) = shell!("ls -R -1 {} | grep '\\.md' | wc -l", self.root) {
            res.trim().parse::<usize>().unwrap_or(0)
        } else {
            0
        }
    }

    /// Find a single wiki page by name.
    pub fn find(&self, name: &str) -> Option<Page> {
        self.pages()
            .unwrap_or_else(|_| vec![])
            .into_iter()
            .find(|p| p.name() == name)
    }

    /// Check if a wiki page exists by path.
    pub fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    /// All the wiki pages, in alphabetical order.
    pub fn pages(&self) -> Result<Vec<Page>> {
        Ok(shell!("find {} -type f -name '*.md' | sort", self.root)?
            .trim()
            .split('\n')
            .map(|line| Page::new(&self.root, line.trim().to_string()))
            .collect())
    }

    /// All the wiki page names, in alphabetical order.
    pub fn names(&self) -> Result<Vec<String>> {
        let mut names: Vec<_> = self.pages()?.iter().map(|p| p.name().to_string()).collect();
        names.sort();
        Ok(names)
    }

    /// All the wiki page titles, in alphabetical order.
    pub fn titles(&self) -> Result<Vec<String>> {
        let mut names: Vec<_> = self.pages()?.iter().map(|p| p.title()).collect();
        names.sort();
        Ok(names)
    }

    /// Recently modified wiki pages.
    pub fn recent(&self) -> Result<Vec<Page>> {
        let out = shell!(
            r#"git --git-dir={}/.git log --pretty=format: --name-only -n 30 | grep "\.md\$""#,
            self.root
        )?;
        let mut pages = vec![];
        let mut seen = HashMap::new();
        for path in out.split("\n") {
            if seen.get(path).is_some() || path == ".md" || path.is_empty() {
                // TODO: .md hack
                continue;
            } else {
                pages.push(Page::new(&self.root, path));
                seen.insert(path, true);
            }
        }
        Ok(pages)
    }

    /// All the tags used, in alphabetical order.
    pub fn tags(&self) -> Result<Vec<String>> {
        let out = match shell!(
            "grep --exclude-dir .git -E -h -o -r '#(\\w+)' {} | sort | uniq",
            self.root
        ) {
            Err(e) => {
                eprintln!("EGREP ERROR: {}", e);
                return Err(e);
            }
            Ok(out) => out,
        };

        Ok(out
            .split('\n')
            .filter_map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s[1..].to_string())
                }
            })
            .collect::<Vec<_>>())
    }

    // Don't include the '#' when you search, eg pass in "hashtag" to
    // search for #hashtag.
    pub fn find_pages_with_tag(&self, tag: &str) -> Result<Vec<Page>> {
        let tag = if tag.starts_with('#') {
            tag.to_string()
        } else {
            format!("#{}", tag)
        };

        let out = shell!("grep --exclude-dir .git -l -r '{}' {}", tag, self.root)?;
        Ok(out
            .split("\n")
            .filter_map(|line| {
                if !line.is_empty() {
                    Some(Page::new(&self.root, line.split(':').next().unwrap_or("?")))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }

    /// Create a new wiki page on disk. Name should be the title, such
    /// as "Linux Laptops" - it'll get converted to linux_laptops.md.
    pub fn create(&self, name: &str, body: &str) -> Result<Page> {
        let path = self.pathify(name);
        if self.exists(&path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Already Exists: {}", path),
            ));
        }
        // mkdir -p
        if path.contains('/') {
            if let Some(dir) = Path::new(&path).parent() {
                fs::create_dir_all(&dir.display().to_string())?;
            }
        }
        let mut file = fs::File::create(&path)?;
        write!(file, "{}", body)?;
        Ok(Page::new(&self.root, path))
    }

    /// Save a page to disk. Doesn't track renames, just content
    /// changes for now.
    pub fn update(&self, name: &str, body: &str) -> Result<Page> {
        let path = self.pathify(name);
        if !self.exists(&path) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Doesn't exist: {}", path),
            ));
        }
        let af = AtomicFile::new(&path, AllowOverwrite);
        af.write(|f| f.write_all(body.as_bytes()))?;
        Ok(Page::new(&self.root, path))
    }

    /// Convert a wiki page name or file path to cleaned up path.
    /// Ex: "Test Results" -> "test_results"
    fn pathify(&self, path: &str) -> String {
        let path = if path.ends_with(".html") && !path.starts_with("html/") {
            format!("html/{}", path)
        } else {
            path.to_string()
        };
        format!(
            "{}{}.md",
            self.root,
            path.to_lowercase()
                .trim_start_matches('/')
                .replace("..", ".")
                .replace(" ", "_")
                .chars()
                .filter(|&c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/')
                .collect::<String>()
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_len() {
        let db = DB::new("./wiki/");
        assert_eq!(5, db.len());
        assert_eq!(false, db.is_empty());

        let db = DB::new("./src/");
        assert_eq!(0, db.len());
        assert_eq!(true, db.is_empty());
    }

    #[test]
    fn test_pages() {
        let db = DB::new("./wiki/");
        let pages = db.pages().unwrap();
        assert_eq!("TODO", pages[0].name());
        assert_eq!("TODO", pages[0].title());
        assert_eq!("keyboard_shortcuts", pages[1].name());
        assert_eq!("Keyboard Shortcuts", pages[1].title());
    }
}
