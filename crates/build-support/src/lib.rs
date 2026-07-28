use std::path::{Path, PathBuf};

pub const LIBCURL_IMPERSONATE_VERSION: &str = "1.5.6";

pub fn find_library() -> Option<PathBuf> {
    find_from_environment()
        .or_else(find_system_library)
}

fn find_from_environment() -> Option<PathBuf> {
    let directory = std::env::var_os("LIBCURL_IMPERSONATE_DIR")?;
    let path = PathBuf::from(directory);

    if contains_library(&path) {
        Some(path)
    } else {
        None
    }
}

fn find_system_library() -> Option<PathBuf> {
    let path: PathBuf = PathBuf::from("/usr/local/lib");

    if contains_library(&path) {
        Some(path)
    } else {
        None
    }
}

fn contains_library(directory: &Path) -> bool {
    directory.join("libcurl-impersonate-chrome.so").exists()
}
