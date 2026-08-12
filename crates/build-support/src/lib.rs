use std::{
    env,
    path::{Path, PathBuf},
};

pub const LIBCURL_IMPERSONATE_VERSION: &str = "1.5.6";

pub fn cache_directory() -> Option<PathBuf> {
    env::var_os("HOME").map(|home_directory| {
        PathBuf::from(home_directory)
            .join(".cache")
            .join("impersonate-rs")
            .join("libcurl-impersonate")
            .join(LIBCURL_IMPERSONATE_VERSION)
    })
}

pub fn find_library() -> Option<PathBuf> {
    find_from_environment()
        .or_else(find_from_cache)
        .or_else(find_system_library)
}

fn find_from_environment() -> Option<PathBuf> {
    let directory = std::env::var_os("LIBCURL_IMPERSONATE_DIR")?;
    let path = PathBuf::from(directory);
    contains_library(&path).then_some(path)
}

fn find_from_cache() -> Option<PathBuf> {
    let path = cache_directory()?;
    contains_library(&path).then_some(path)
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
