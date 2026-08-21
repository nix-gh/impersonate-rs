use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub const LIBCURL_IMPERSONATE_VERSION: &str = "1.5.6";
const LINK_LIBRARY_NAME: &str = "libcurl-impersonate.so";
pub const LIBCURL_IMPERSONATE_RELEASE_URL: &str =
    "https://github.com/lexiforest/curl-impersonate/releases/download";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibcurlTarget {
    X86_64LinuxGnu,
    Aarch64LinuxAndroid,
}

pub fn runtime_library_name(target: LibcurlTarget) -> &'static str {
    match target {
        LibcurlTarget::X86_64LinuxGnu => "libcurl-impersonate.so.4",
        LibcurlTarget::Aarch64LinuxAndroid => "libcurl-impersonate.so",
    }
}

impl LibcurlTarget {
    pub fn detect_host() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Self::X86_64LinuxGnu),
            ("android", "aarch64") => Some(Self::Aarch64LinuxAndroid),
            _ => None,
        }
    }

    pub fn release_target(self) -> &'static str {
        match self {
            Self::X86_64LinuxGnu => "x86_64-linux-gnu",
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

    pub fn impersonate_runtime_library_path(self) -> Option<PathBuf> {
        match self {
            Self::X86_64LinuxGnu => None,
            Self::Aarch64LinuxAndroid => {
                env::var_os("PREFIX").map(|prefix| PathBuf::from(prefix).join("lib"))
            }
        }
    }

    pub fn set_impersonate_runtime_library_path(&self, library_path: &Path) -> Result<(), String> {
        let Some(runtime_library_path) = self.impersonate_runtime_library_path() else {
            return Ok(());
        };

        let status = Command::new("patchelf")
            .args(["--set-rpath"])
            .arg(&runtime_library_path)
            .arg(library_path)
            .status()
            .map_err(|error| format!("failed to execute patchelf: {error}"))?;

        if !status.success() {
            return Err(format!(
                "patchelf failed to set runtime library path on {}",
                library_path.display()
            ));
        }

        Ok(())
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

pub fn contains_compatible_library(directory: &Path, target: LibcurlTarget) -> bool {
    directory.join(LINK_LIBRARY_NAME).exists()
        && directory.join(runtime_library_name(target)).exists()
}

fn find_from_cache(target: LibcurlTarget) -> Option<PathBuf> {
    let path = cache_directory(LibcurlTarget::detect_host()?)?;
    contains_compatible_library(&path, target).then_some(path)
}

fn find_system_library(target: LibcurlTarget) -> Option<PathBuf> {
    let path: PathBuf = PathBuf::from("/usr/local/lib");

    if contains_compatible_library(&path, target) {
        Some(path)
    } else {
        None
    }
}

pub fn find_library() -> Result<Option<PathBuf>, String> {
    let target =
        LibcurlTarget::detect_host().ok_or_else(|| "unsupported host platform".to_owned())?;
    if let Some(directory) = std::env::var_os("LIBCURL_IMPERSONATE_DIR") {
        let path = PathBuf::from(directory);

        if contains_compatible_library(&path, target) {
            return Ok(Some(path));
        }

        return Err(format!(
            "LIBCURL_IMPERSONATE_DIR does not contain a compatible libcurl-impersonate installation: {}",
            path.display()
        ));
    }

    Ok(find_from_cache(target).or_else(|| find_system_library(target)))
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

    if !contains_compatible_library(&cache_directory, target) {
        download_archive(target.download_url().as_str(), &archive_path)?;
        extract_archive(&archive_path, &cache_directory, target)?;
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

fn extract_archive(
    archive_path: &Path,
    destination: &Path,
    target: LibcurlTarget,
) -> Result<(), String> {
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

    if !contains_compatible_library(destination, target) {
        return Err(format!(
            "libcurl-impersonate was extracted but the expected library was not found in {}",
            destination.display()
        ));
    }

    Ok(())
}

pub fn find_or_install_library() -> Result<std::path::PathBuf, String> {
    if let Some(library_directory) = find_library()? {
        return Ok(library_directory);
    }

    install_libcurl()
}

pub fn target_directory() -> Option<PathBuf> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR")?);

    out_dir.ancestors().nth(3).map(Path::to_path_buf)
}

pub fn android_ca_bundle() -> Option<PathBuf> {
    LibcurlTarget::detect_host()
        .filter(|target| *target == LibcurlTarget::Aarch64LinuxAndroid)
        .and_then(|_| {
            env::var_os("PREFIX")
                .map(PathBuf::from)
                .map(|prefix| prefix.join("etc/tls/cert.pem"))
        })
}