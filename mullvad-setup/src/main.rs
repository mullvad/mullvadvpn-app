use clap::{crate_authors, crate_description, crate_name, SubCommand};
use std::process;
use talpid_core::firewall::{self, Firewall, FirewallArguments};
use talpid_types::ErrorExt;
use parity_tokio_ipc::Endpoint as IpcEndpoint;
use tower::service_fn;
use tonic::{
    self,
    transport::{Endpoint, Uri},
};

mod proto {
    tonic::include_proto!("mullvad_daemon.management_interface");
}
use proto::management_service_client::ManagementServiceClient;

pub const PRODUCT_VERSION: &str = include_str!(concat!(env!("OUT_DIR"), "/product-version.txt"));

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Failed to connect to daemon")]
    DaemonConnect(#[error(source)] tonic::transport::Error),

    #[error(display = "RPC call failed")]
    DaemonRpcError(#[error(source)] tonic::Status),

    #[error(display = "This command cannot be run if the daemon is active")]
    DaemonIsRunning,

    #[error(display = "Firewall error")]
    FirewallError(#[error(source)] firewall::Error),
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let subcommands = vec![
        SubCommand::with_name("prepare-restart")
            .about("Move a running daemon into a blocking state and save its target state"),
        SubCommand::with_name("reset-firewall")
            .about("Remove any firewall rules introduced by the daemon"),
    ];

    let app = clap::App::new(crate_name!())
        .version(PRODUCT_VERSION)
        .author(crate_authors!())
        .about(crate_description!())
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&[
            clap::AppSettings::DisableHelpSubcommand,
            clap::AppSettings::VersionlessSubcommands,
        ])
        .subcommands(subcommands);

    let matches = app.get_matches();
    let result = match matches.subcommand_name().expect("Subcommand has no name") {
        "prepare-restart" => prepare_restart().await,
        "reset-firewall" => reset_firewall().await,
        _ => unreachable!("No command matched"),
    };

    if let Err(e) = result {
        eprintln!("{}", e.display_chain());
        process::exit(1);
    }
}

async fn prepare_restart() -> Result<(), Error> {
    let mut rpc = new_grpc_client().await?;
    rpc.prepare_restart(()).await.map_err(Error::DaemonRpcError)?;
    Ok(())
}

async fn reset_firewall() -> Result<(), Error> {
    // Ensure that the daemon isn't running
    if let Ok(_) = new_grpc_client().await {
        return Err(Error::DaemonIsRunning);
    }

    let mut firewall = Firewall::new(FirewallArguments {
        initialize_blocked: false,
        allow_lan: None,
    })
    .map_err(Error::FirewallError)?;

    firewall.reset_policy().map_err(Error::FirewallError)
}

pub async fn new_grpc_client() -> Result<ManagementServiceClient<tonic::transport::Channel>, Error> {
    let ipc_path = mullvad_paths::get_rpc_socket_path();

    // The URI will be ignored
    let channel = Endpoint::from_static("lttp://[::]:50051")
        .connect_with_connector(service_fn(move |_: Uri| {
            IpcEndpoint::connect(ipc_path.clone())
        }))
        .await
        .map_err(Error::DaemonConnect)?;

    Ok(ManagementServiceClient::new(channel))
}
