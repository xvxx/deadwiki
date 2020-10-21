use std::{fs, os::unix::fs::PermissionsExt};

/// Is the file at the given path `chmod +x`?
pub fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Encode just a few basic characters into HTML entities.
pub fn html_encode(html: &str) -> String {
    html.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
