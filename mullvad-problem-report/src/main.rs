#![deny(rust_2018_idioms)]

use clap::Parser;
use mullvad_problem_report::{collect_report, Error};
use std::{
    env,
    path::{Path, PathBuf},
    process,
};
use talpid_types::ErrorExt;

fn main() {
    process::exit(match run() {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("{}", error.display_chain());
            1
        }
    })
}

#[derive(Debug, Parser)]
#[command(author, version = mullvad_version::VERSION, about, long_about = None)]
#[command(
    arg_required_else_help = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
enum Cli {
    /// Collect problem report to a single file
    Collect {
        /// The destination path for saving the collected report
        #[arg(required = true, long, short = 'o')]
        output: PathBuf,
        /// Paths to additional log files to be included
        extra_logs: Vec<PathBuf>,
        /// List of strings to remove from the report
        #[arg(long)]
        redact: Vec<String>,
    },

    /// Send collected problem report
    Send {
        /// Path to a previously collected report file
        #[arg(required = true, long, short = 'r')]
        report: PathBuf,
        /// Email to attach to the problem report
        #[arg(long, short = 'e')]
        email: Option<String>,
        /// Message to include in the problem report
        #[arg(long, short = 'm')]
        message: Option<String>,
    },
}

fn run() -> Result<(), Error> {
    env_logger::init();

    match Cli::parse() {
        Cli::Collect {
            output,
            extra_logs,
            redact,
        } => {
            let expanded_output_path = output.canonicalize().unwrap_or_else(|_| output.to_owned());

            collect_report(&extra_logs, &expanded_output_path, redact)?;

            println!(
                "Problem report written to {}",
                expanded_output_path.display()
            );
            println!();
            println!("Send the problem report to support via the send subcommand. See:");
            println!(" $ {} send --help", env::args().next().unwrap());
        }
        Cli::Send {
            report,
            email,
            message,
        } => {
            send_problem_report(
                &email.unwrap_or_default(),
                &message.unwrap_or_default(),
                &report,
            )?;
        }
    }

    Ok(())
}

fn send_problem_report(
    user_email: &str,
    user_message: &str,
    report_path: &Path,
) -> Result<(), Error> {
    let cache_dir = mullvad_paths::get_cache_dir().map_err(Error::ObtainCacheDirectory)?;
    mullvad_problem_report::send_problem_report(user_email, user_message, report_path, &cache_dir)
        .map_err(|error| {
            eprintln!("{}", error.display_chain());
            error
        })?;

    println!("Problem report sent");
    Ok(())
}
