use {
    crate::render::pathify,
    rust_embed::RustEmbed,
    std::{io, str},
};

#[derive(RustEmbed)]
#[folder = "static/"]
pub struct Asset;

/// Does the asset exist on disk? `path` is the relative path,
/// ex: asset_exists("index.html") checks for "./static/index.html"
/// (or in the embedded fs, in release mode).
pub fn exists(path: &str) -> bool {
    let path = pathify(path);
    Asset::get(&path).is_some()
}

/// Like fs::read_to_string(), but with an asset.
pub fn to_string(path: &str) -> Result<String, io::Error> {
    if let Some(asset) = Asset::get(&path) {
        if let Ok(utf8) = str::from_utf8(asset.as_ref()) {
            return Ok(utf8.to_string());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} not found", path),
    ))
}
