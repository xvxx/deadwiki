use crate::Page;
use walkdir::WalkDir;

pub type Result<T> = std::result::Result<T, std::io::Error>;

pub struct DB {
    root: String,
}

impl DB {
    /// Create a new DB object. Should only have one per run.
    pub fn new(root: String) -> DB {
        DB { root }
    }

    /// All the wiki pages, in alphabetical order.
    pub fn pages(&self) -> Result<Vec<Page>> {
        let mut pages = vec![];

        for entry in WalkDir::new(&self.root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir()
                && entry.file_name().to_str().unwrap_or("").ends_with(".md")
            {
                let path = entry.path().display().to_string();
                let name = path
                    .trim_start_matches(&self.root)
                    .trim_start_matches('.')
                    .trim_start_matches('/')
                    .trim_end_matches(".md")
                    .to_lowercase();
                if !name.is_empty() {
                    pages.push(Page::new(&name, &path));
                }
            }
        }

        Ok(pages)
    }

    /// All the wiki page titles, in alphabetical order.
    pub fn titles(&self) -> Result<Vec<String>> {
        let mut names: Vec<_> = self.pages()?.iter().map(|p| p.name.to_string()).collect();
        names.sort();
        Ok(names)
    }

    /// All the tags used, in alphabetical order.
    pub fn tags(&self) -> Result<Vec<String>> {
        let out = match shell!(
            "grep --exclude-dir .git -E -h -o -r '#(\\w+)' {} | sort | uniq",
            self.root
        ) {
            Err(e) => {
                eprintln!("EGREP ERROR: {}", e);
                return vec![];
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
}
