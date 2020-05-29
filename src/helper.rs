use {
    std::{fs, io, os::unix::fs::PermissionsExt, path::Path},
    vial::asset,
};

/// Capitalize the first letter of a string.
pub fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap_or('?').to_uppercase(),
        &s.chars().skip(1).collect::<String>()
    )
}

/// some_page -> Some Page
pub fn wiki_path_to_title(path: &str) -> String {
    path.trim_start_matches('/')
        .trim_end_matches(".md")
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

/// Return the <nav> for a page
pub fn nav(current_path: &str) -> Result<String, io::Error> {
    Ok(asset::to_string("html/nav.html")?.replace("{current_path}", current_path))
}

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
    format!("{}/{}.md", crate::wiki_root(), pathify(path))
}

/// All the wiki pages, in alphabetical order.
pub fn page_names() -> Vec<String> {
    let mut names = vec![];
    let root = crate::wiki_root();

    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() && entry.file_name().to_str().unwrap_or("").ends_with(".md")
        {
            let name = entry.path().display().to_string();
            let name = name
                .trim_start_matches(&root)
                .trim_start_matches('.')
                .trim_start_matches('/')
                .trim_end_matches(".md");
            if !name.is_empty() {
                names.push(name.to_lowercase());
            }
        }
    }

    names.sort();
    names
}
