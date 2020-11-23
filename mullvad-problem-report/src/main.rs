#![deny(rust_2018_idioms)]

use clap::{crate_authors, crate_name};
use mullvad_problem_report::{collect_report, metadata, send_problem_report, Error};
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
        .version(metadata::PRODUCT_VERSION)
        .author(crate_authors!())
        .about("Mullvad VPN problem report tool. Collects logs and sends them to Mullvad support.")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&[
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::VersionlessSubcommands,
        ])
        .subcommand(
            clap::SubCommand::with_name("collect")
                .about("Collect problem report")
                .arg(
                    clap::Arg::with_name("output")
                        .help("The destination path for saving the collected report.")
                        .long("output")
                        .short("o")
                        .value_name("PATH")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("extra_logs")
                        .help("Paths to additional log files to be included.")
                        .multiple(true)
                        .value_name("EXTRA LOGS")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::with_name("redact")
                        .help("List of words and expressions to remove from the report")
                        .long("redact")
                        .value_name("PHRASE")
                        .multiple(true)
                        .takes_value(true),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("send")
                .about("Send collected problem report")
                .arg(
                    clap::Arg::with_name("report")
                        .long("report")
                        .short("r")
                        .help("The path to previously collected report file.")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    clap::Arg::with_name("email")
                        .long("email")
                        .short("e")
                        .help("Reporter's email")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    clap::Arg::with_name("message")
                        .long("message")
                        .short("m")
                        .help("Reporter's message")
                        .takes_value(true)
                        .required(false),
                ),
        );

    let matches = app.get_matches();

    if let Some(collect_matches) = matches.subcommand_matches("collect") {
        let redact_custom_strings = collect_matches
            .values_of_lossy("redact")
            .unwrap_or_else(Vec::new);
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
        println!("");
        println!("Send the problem report to support via the send subcommand. See:");
        println!(" $ {} send --help", env::args().next().unwrap());
        Ok(())
    } else if let Some(send_matches) = matches.subcommand_matches("send") {
        let report_path = Path::new(send_matches.value_of_os("report").unwrap());
        let user_email = send_matches.value_of("email").unwrap_or("");
        let user_message = send_matches.value_of("message").unwrap_or("");
        let resource_dir = mullvad_paths::get_resource_dir();
        send_problem_report(user_email, user_message, report_path, &resource_dir)
    } else {
        unreachable!("No sub command given");
    }
}
