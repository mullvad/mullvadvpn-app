// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

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

use std::fs::File;
use std::io::Read;

mod cli;


error_chain!{}

quick_main!(run);

fn run() -> Result<()> {
    env_logger::init().chain_err(|| "Failed to bootstrap logging system")?;

    let matches = cli::get_matches();
    if let Some(matches) = matches.subcommand_matches("account") {
        cmd_account(matches)
    } else {
        unreachable!("No subcommand matches.")
    }
}

fn cmd_account(matches: &clap::ArgMatches) -> Result<()> {
    if let Some(matches) = matches.subcommand_matches("set") {
        let token = matches.value_of("token").unwrap();
        call_rpc("set_account", &[token]).map(
            |_| {
                println!("Mullvad account {} set", token);
            },
        )
    } else if let Some(_matches) = matches.subcommand_matches("get") {
        match call_rpc("get_account", &[] as &[u8; 0])? {
            serde_json::Value::String(token) => println!("Mullvad account: {:?}", token),
            serde_json::Value::Null => println!("No account configured"),
            _ => bail!("Unable to fetch account token"),
        }
        Ok(())
    } else {
        unreachable!("No account command given");
    }
}

fn call_rpc<T>(method: &str, args: &T) -> Result<serde_json::Value>
    where T: serde::Serialize
{
    let address = read_rpc_address()?;
    info!("Using RPC address {}", address);
    let mut rpc_client = talpid_ipc::WsIpcClient::new(address)
        .chain_err(|| "Unable to create RPC client")?;
    rpc_client.call(method, args).chain_err(|| "Unable to call RPC method")
}

fn read_rpc_address() -> Result<String> {
    for path in &["./.mullvad_rpc_address", "../.mullvad_rpc_address"] {
        debug!("Trying to read RPC address at {}", path);
        let mut address = String::new();
        if let Ok(_) = File::open(path).and_then(|mut file| file.read_to_string(&mut address)) {
            return Ok(address);
        }
    }
    bail!("Unable to read RPC address");
}
