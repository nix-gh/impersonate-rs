use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{fs, path::Path, process};
use build_support::cache_directory;

#[derive(Parser)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    InstallLibcurl,
}

fn main() -> Result<()> {
    let command_line = CommandLine::parse();

    match command_line.command {
        Command::InstallLibcurl => install_libcurl(),
    }
}

fn download_archive(download_url: &str, archive_path: &Path) -> Result<()> {
    let temporary_path = archive_path.with_extension("download");

    let status = process::Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error"])
        .arg("--output")
        .arg(&temporary_path)
        .arg(download_url)
        .status()
        .context("failed to execute curl")?;

    if !status.success() {
        let _ = fs::remove_file(&temporary_path);
        anyhow::bail!("curl failed with status {status}");
    }

    fs::rename(&temporary_path, archive_path).with_context(|| {
        format!(
            "failed to move downloaded archive to {}",
            archive_path.display()
        )
    })?;

    Ok(())
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<()> {
    let status = process::Command::new("tar")
        .args(["--extract", "--gzip"])
        .arg("--file")
        .arg(archive_path)
        .arg("--directory")
        .arg(destination)
        .status()
        .context("failed to execute tar")?;

    if !status.success() {
        anyhow::bail!("tar failed with status {status}");
    }
    if !build_support::contains_library(&destination) {
        anyhow::bail!(
            "libcurl-impersonate was extracted but the expected library was not found in {}",
            destination.display()
        );
    }

    Ok(())
}

fn install_libcurl() -> Result<()> {
    let target =
        build_support::LibcurlTarget::detect_host().context("unsupported host platform")?;

    let cache_directory =
        build_support::cache_directory(target).context("HOME environment variable is not set")?;

    let archive_path = build_support::cache_archive_path(target)
        .context("HOME environment variable is not set")?;

    println!(
        "Installing libcurl-impersonate v{}...",
        build_support::LIBCURL_IMPERSONATE_VERSION
    );
    println!("Target: {}", target.release_target());

    if build_support::contains_library(&cache_directory) {
        println!("Using cached installation: {}", cache_directory.display());
        return Ok(());
    }

    fs::create_dir_all(&cache_directory).with_context(|| {
        format!(
            "failed to create cache directory {}",
            cache_directory.display()
        )
    })?;

    if !archive_path.exists() {
        let download_url = target.download_url();
        println!("Downloading: {}", download_url);
        download_archive(download_url.as_str(), &archive_path)?;
        println!("Downloaded: {}", archive_path.display());
    }

    extract_archive(&archive_path, &cache_directory)?;

    println!("Installed to {}", cache_directory.display());

    Ok(())
}
