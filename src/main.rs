use std::{env::set_current_dir, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

mod util;

#[derive(Parser)]
struct Args {
    /// Path to config file
    #[arg(default_value_t = String::from("./droplet.toml"))]
    config: String,

    /// Don't update DNS
    #[arg(long)]
    no_dns: bool,

    /// Don't pull changes from remote
    #[arg(long)]
    no_pull: bool,

    /// Don't push changes to remote
    #[arg(long)]
    no_push: bool,

    /// Alias for --no-pull --no-push
    #[arg(long)]
    no_sync: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config_path = PathBuf::from(args.config)
        .canonicalize()
        .context("couldn't canonicalize config path")?;

    let config_parent_dir = config_path
        .parent()
        .context("couldn't determine parent directory")?;

    set_current_dir(config_parent_dir).context("failed changing directory")?;
    println!("changed directory to {}", config_parent_dir.display());

    let config = util::get_config(&config_path).context(format!(
        "failed loading config file {}",
        &config_path.display()
    ))?;
    println!("configuration loaded");

    if !args.no_dns {
        let response = util::update_dns(&config);

        match response {
            Ok(r) => {
                for line in r.lines() {
                    println!("dns: {line}");
                }
            }
            Err(e) => {
                eprintln!();
                eprintln!("warning:");
                eprintln!("    failed to update DNS record");
                eprintln!("    your service will only be accessible via your public IP address");
                eprintln!("    source of error:");
                for line in e.to_string().lines() {
                    eprintln!("        {line}");
                }
                eprintln!()
            }
        }
    }

    if !args.no_pull && !args.no_sync {
        util::sync_pull(&config)?;
        println!("pulled changes from remote");
    }

    let service = util::start_service(&config).context("failed starting service")?;
    println!("service started");

    let output = service
        .wait_with_output()
        .context("failed waiting for service process")?;

    let exit_code = output.status.code();

    println!(
        "service exited with code {}",
        exit_code
            .map(|c| c.to_string())
            .unwrap_or("<none>".to_string())
    );

    if !args.no_push && !args.no_sync {
        util::sync_push(&config)?;
        println!("pushed changes to remote");
    }

    Ok(())
}
