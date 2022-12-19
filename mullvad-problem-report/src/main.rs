#![deny(rust_2018_idioms)]

use clap::{crate_authors, crate_name};
use mullvad_problem_report::{collect_report, Error};
use std::{env, path::Path, process};
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

fn run() -> Result<(), Error> {
    env_logger::init();
    let app = clap::App::new(crate_name!())
        .version(mullvad_version::VERSION)
        .author(crate_authors!())
        .about("Mullvad VPN problem report tool. Collects logs and sends them to Mullvad support.")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_setting(clap::AppSettings::DisableHelpSubcommand)
        .global_setting(clap::AppSettings::DisableVersionFlag)
        .subcommand(
            clap::App::new("collect")
                .about("Collect problem report")
                .arg(
                    clap::Arg::new("output")
                        .help("The destination path for saving the collected report.")
                        .long("output")
                        .short('o')
                        .value_name("PATH")
                        .allow_invalid_utf8(true)
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("extra_logs")
                        .help("Paths to additional log files to be included.")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .value_name("EXTRA LOGS")
                        .allow_invalid_utf8(true)
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::new("redact")
                        .help("List of words and expressions to remove from the report")
                        .long("redact")
                        .value_name("PHRASE")
                        .multiple_occurrences(true)
                        .multiple_values(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            clap::App::new("send")
                .about("Send collected problem report")
                .arg(
                    clap::Arg::new("report")
                        .long("report")
                        .short('r')
                        .help("The path to previously collected report file.")
                        .allow_invalid_utf8(true)
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::new("email")
                        .long("email")
                        .short('e')
                        .help("Reporter's email")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::new("message")
                        .long("message")
                        .short('m')
                        .help("Reporter's message")
                        .takes_value(true)
                        .required(false),
                ),
        );

    let matches = app.get_matches();

    if let Some(collect_matches) = matches.subcommand_matches("collect") {
        let redact_custom_strings = collect_matches
            .values_of_t("redact")
            .unwrap_or_else(|_| vec![]);
        let extra_logs = collect_matches
            .values_of_os("extra_logs")
            .map(|os_values| os_values.map(Path::new).collect())
            .unwrap_or_else(Vec::new);
        let output_path = Path::new(collect_matches.value_of_os("output").unwrap());
        collect_report(&extra_logs, output_path, redact_custom_strings)?;

        let expanded_output_path = output_path
            .canonicalize()
            .unwrap_or_else(|_| output_path.to_owned());
        println!(
            "Problem report written to {}",
            expanded_output_path.display()
        );
        println!();
        println!("Send the problem report to support via the send subcommand. See:");
        println!(" $ {} send --help", env::args().next().unwrap());
        Ok(())
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = Path::new(send_matches.value_of_os("report").unwrap());
        let user_email = send_matches.value_of("email").unwrap_or("");
        let user_message = send_matches.value_of("message").unwrap_or("");
        send_problem_report(user_email, user_message, report_path)
    } else {
        unreachable!("No sub command given");
    }
}

fn send_problem_report(
    user_email: &str,
    user_message: &str,
    report_path: &Path,
) -> Result<(), Error> {
    let cache_dir = mullvad_paths::get_cache_dir().map_err(Error::ObtainCacheDirectory)?;
    mullvad_problem_report::send_problem_report(user_email, user_message, report_path, &cache_dir)
        .map(|()| println!("Problem report sent"))
        .map_err(|error| {
            eprintln!("{}", error.display_chain());
            error
        })
}
