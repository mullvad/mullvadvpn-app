// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate mullvad_types;
extern crate talpid_ipc;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate serde;
extern crate serde_json;

mod rpc;
mod cmds;


error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")?;

    let commands = cmds::get_commands();

    let app = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequired)
        .subcommands(commands.values().map(|cmd| cmd.clap_subcommand()));

    let app_matches = app.get_matches();
    let (subcommand_name, subcommand_matches) = app_matches.subcommand();
    if let Some(cmd) = commands.get(subcommand_name) {
        cmd.run(subcommand_matches.expect("No command matched"))
    } else {
        unreachable!("No command matched");
    }
}

pub trait Command {
    fn name(&self) -> &'static str;

    fn clap_subcommand(&self) -> clap::App<'static, 'static>;

    fn run(&self, matches: &clap::ArgMatches) -> Result<()>;
}
