use {
    std::{fs, io, os::unix::fs::PermissionsExt},
    vial::asset,
};

/// Is the file at the given path `chmod +x`?
pub fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Return the <nav> for a page
pub fn nav(current_path: &str) -> Result<String, io::Error> {
    let new_link = if current_path.contains('/') {
        format!(
            "/new?name={}/",
            current_path
                .split('/')
                .take(current_path.matches('/').count())
                .collect::<Vec<_>>()
                .join("/")
        )
    } else {
        "/new".to_string()
    };
    Ok(asset::to_string("html/nav.html")?
        .replace("{current_path}", current_path)
        .replace("{new_link}", &new_link))
}
