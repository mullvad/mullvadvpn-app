use anyhow::{Context, anyhow, bail};
use clap::Parser;
use nix::unistd::{execvp, getgid, getuid, setgid, setuid};
use nullvad::{do_in_namespace, open_namespace_file};
use std::ffi::CString;

/// Execute a command inside the "nullvad" network namespace
#[derive(Parser)]
struct Opt {
    command: Vec<CString>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    // Enter the namespace
    let netns_fd = open_namespace_file().await?;
    do_in_namespace(netns_fd, move || {
        let args = opt.command;
        let Some(program) = args.first() else {
            bail!("No command specified");
        };

        // Drop privileges
        let real_uid = getuid();
        let real_gid = getgid();
        setuid(real_uid).context("Failed to drop user privileges")?;
        setgid(real_gid).context("Failed to drop group privileges")?;

        // Launch the process
        let infallible =
            execvp(program, &args).with_context(|| anyhow!("Failed to exec {program:?}"))?;

        match infallible {}
    })
    .await?
}
