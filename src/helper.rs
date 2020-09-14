use {
    crate::wiki_root,
    std::{fs, os::unix::fs::PermissionsExt, path::Path},
};

/// Is the file at the given path `chmod +x`?
pub fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Convert a wiki page name or file path to cleaned up path.
/// Ex: "Test Results" -> "test_results"
pub fn pathify(path: &str) -> String {
    let path = if path.ends_with(".html") && !path.starts_with("html/") {
        format!("html/{}", path)
    } else {
        path.to_string()
    };
    path.to_lowercase()
        .trim_start_matches('/')
        .replace("..", ".")
        .replace(" ", "_")
        .chars()
        .filter(|&c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-' || c == '/')
        .collect::<String>()
}

/// Path of wiki page on disk, if it exists.
/// Ex: page_path("Welcome") -> "wiki/welcome.md"
pub fn page_path(path: &str) -> Option<String> {
    let path = page_disk_path(path);
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// Returns a path on disk to a new wiki page.
/// Nothing if the page already exists.
pub fn new_page_path(path: &str) -> Option<String> {
    if page_path(path).is_none() {
        Some(page_disk_path(path))
    } else {
        None
    }
}

/// Returns a wiki path on disk, regardless of whether it exists.
pub fn page_disk_path(path: &str) -> String {
    format!("{}/{}.md", wiki_root(), pathify(path))
}
