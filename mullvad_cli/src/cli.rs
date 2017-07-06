use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn get_matches() -> ArgMatches<'static> {
    let app = create_app();
    app.clone().get_matches()
}

fn create_app() -> App<'static, 'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("account")
            .about("Control and display information about the configured Mullvad account")
            .setting(AppSettings::SubcommandRequired)
            .subcommand(SubCommand::with_name("set")
                .about("Change the Mullvad account")
                .arg(Arg::with_name("token")
                    .help("The Mullvad account token to configure the daemon with")
                    .required(true)))
            .subcommand(SubCommand::with_name("get")
                .about("Display information about the currently configured account")))
}
