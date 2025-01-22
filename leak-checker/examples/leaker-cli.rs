use clap::{Parser, Subcommand};
use leak_checker::traceroute::TracerouteOpt;

#[derive(Parser)]
pub struct Opt {
    #[clap(subcommand)]
    pub method: LeakMethod,
}

#[derive(Subcommand, Clone)]
pub enum LeakMethod {
    /// Check for leaks by binding to a non-tunnel interface and probing for reachable nodes.
    Traceroute(#[clap(flatten)] TracerouteOpt),

    /// Ask `am.i.mullvad.net` whether you are leaking.
    #[cfg(feature = "am-i-mullvad")]
    AmIMullvad(#[clap(flatten)] leak_checker::am_i_mullvad::AmIMullvadOpt),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Debug)
        .parse_default_env()
        .init();

    let opt = Opt::parse();

    let leak_status = match &opt.method {
        LeakMethod::Traceroute(opt) => leak_checker::traceroute::run_leak_test(opt).await,
        #[cfg(feature = "am-i-mullvad")]
        LeakMethod::AmIMullvad(opt) => leak_checker::am_i_mullvad::run_leak_test(opt).await,
    };

    log::info!("Leak status: {leak_status:#?}");

    Ok(())
}
