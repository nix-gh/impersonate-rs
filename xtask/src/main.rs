use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{fs, path::Path, process};

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

fn install_libcurl() -> Result<()> {
    let target =
        build_support::LibcurlTarget::detect_host().context("unsupported host platform")?;

    let cache_directory =
        build_support::cache_directory().context("HOME environment variable is not set")?;

    let archive_path = build_support::cache_archive_path(target)
        .context("HOME environment variable is not set")?;

    println!(
        "Installing libcurl-impersonate v{}...",
        build_support::LIBCURL_IMPERSONATE_VERSION
    );
    println!("Target: {}", target.release_target());

    if archive_path.exists() {
        println!("Using cached archive: {}", archive_path.display());
        return Ok(());
    }

    fs::create_dir_all(&cache_directory).with_context(|| {
        format!(
            "failed to create cache directory {}",
            cache_directory.display()
        )
    })?;

    println!("Download: {}", target.download_url());

    download_archive(target.download_url().as_str(), &archive_path)?;

    println!("Downloaded: {}", archive_path.display());

    Ok(())
}
