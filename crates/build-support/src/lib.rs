use std::{
    env,
    path::{Path, PathBuf},
};

pub const LIBCURL_IMPERSONATE_VERSION: &str = "1.5.6";
pub const LIBCURL_IMPERSONATE_RELEASE_URL: &str =
    "https://github.com/lexiforest/curl-impersonate/releases/download";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibcurlTarget {
    X86_64LinuxGnu,
    X86_64LinuxAndroid,
    Aarch64LinuxAndroid,
}

impl LibcurlTarget {
    pub fn detect_host() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Self::X86_64LinuxGnu),
            _ => None,
        }
    }

    pub fn release_target(self) -> &'static str {
        match self {
            Self::X86_64LinuxGnu => "x86_64-linux-gnu",
            Self::X86_64LinuxAndroid => "x86_64-linux-android",
            Self::Aarch64LinuxAndroid => "aarch64-linux-android",
        }
    }

    pub fn archive_name(self) -> String {
        format!(
            "libcurl-impersonate-v{}.{}.tar.gz",
            LIBCURL_IMPERSONATE_VERSION,
            self.release_target(),
        )
    }

    pub fn download_url(self) -> String {
        format!(
            "{}/v{}/{}",
            LIBCURL_IMPERSONATE_RELEASE_URL,
            LIBCURL_IMPERSONATE_VERSION,
            self.archive_name(),
        )
    }
}

pub fn cache_directory() -> Option<PathBuf> {
    env::var_os("HOME").map(|home_directory| {
        PathBuf::from(home_directory)
            .join(".cache")
            .join("impersonate-rs")
            .join("libcurl-impersonate")
            .join(LIBCURL_IMPERSONATE_VERSION)
    })
}

pub fn cache_archive_path(target: LibcurlTarget) -> Option<PathBuf> {
    let cache_directory = cache_directory()?;
    Some(cache_directory.join(target.archive_name()))
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
