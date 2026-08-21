use std::{
    fs,
    env,
    path::{Path, PathBuf},
    process::Command,
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

pub fn cache_directory(target: LibcurlTarget) -> Option<PathBuf> {
    env::var_os("HOME").map(|home_directory| {
        PathBuf::from(home_directory)
            .join(".cache")
            .join("impersonate-rs")
            .join("libcurl-impersonate")
            .join(target.release_target())
            .join(LIBCURL_IMPERSONATE_VERSION)
    })
}

pub fn cache_archive_path(target: LibcurlTarget) -> Option<PathBuf> {
    let cache_directory = cache_directory(target)?;
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
    let path = cache_directory(LibcurlTarget::detect_host()?)?;
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

pub fn contains_library(directory: &Path) -> bool {
    directory.join("libcurl-impersonate.so").exists()
}

fn install_libcurl() -> Result<std::path::PathBuf, String> {
    let target =
        LibcurlTarget::detect_host().ok_or_else(|| "unsupported host platform".to_owned())?;

    let cache_directory =
        cache_directory(target).ok_or_else(|| "HOME environment variable is not set".to_owned())?;

    let archive_path = cache_archive_path(target)
        .ok_or_else(|| "HOME environment variable is not set".to_owned())?;

    fs::create_dir_all(&cache_directory)
        .map_err(|error| format!("failed to create cache directory: {error}"))?;

    if !contains_library(&cache_directory) {
        download_archive(target.download_url().as_str(), &archive_path)?;
        extract_archive(&archive_path, &cache_directory)?;
    }

    Ok(cache_directory)
}

fn download_archive(download_url: &str, archive_path: &Path) -> Result<(), String> {
    if archive_path.exists() {
        return Ok(());
    }

    let temporary_path = archive_path.with_extension("download");

    let status = Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error"])
        .arg("--output")
        .arg(&temporary_path)
        .arg(download_url)
        .status()
        .map_err(|error| format!("failed to execute curl: {error}"))?;

    if !status.success() {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!("curl failed with status {status}"));
    }

    fs::rename(&temporary_path, archive_path)
        .map_err(|error| format!("failed to store downloaded archive: {error}"))?;

    Ok(())
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let status = Command::new("tar")
        .args(["--extract", "--gzip"])
        .arg("--file")
        .arg(archive_path)
        .arg("--directory")
        .arg(destination)
        .status()
        .map_err(|error| format!("failed to execute tar: {error}"))?;

    if !status.success() {
        return Err(format!("tar failed with status {status}"));
    }

    if !contains_library(destination) {
        return Err(format!(
            "libcurl-impersonate was extracted but the expected library was not found in {}",
            destination.display()
        ));
    }

    Ok(())
}

pub fn find_or_install_library() -> Result<std::path::PathBuf, String> {
    if let Some(library_directory) = find_library() {
        return Ok(library_directory);
    }

    install_libcurl()
}
