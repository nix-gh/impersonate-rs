use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

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

fn install_libcurl() -> Result<()> {
    let target =
        build_support::LibcurlTarget::detect_host().context("unsupported host platform")?;

    println!(
        "Installing libcurl-impersonate v{}...",
        build_support::LIBCURL_IMPERSONATE_VERSION
    );
    println!("Target: {}", target.release_target());
    println!("Download: {}", target.download_url());

    Ok(())
}
