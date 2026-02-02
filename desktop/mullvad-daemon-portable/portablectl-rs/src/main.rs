use std::path::Path;

use anyhow::Context;
use clap::Parser;

use crate::bus::Change;

mod bus;

#[derive(Parser)]
struct Opt {
    #[clap(long, env = "RUST_LOG", default_value = "info")]
    log_filter: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    List,
    Attach {
        image: String,
        #[clap(long, default_value = "default")]
        profile: String,
    },
    Detach {
        image: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();

    let fmt_subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(&opt.log_filter)
        .finish();
    tracing::subscriber::set_global_default(fmt_subscriber)
        .context("Failed to initialize tracing subscriber")?;

    tracing::info!("Connecting to dbus");
    let connection = zbus::connection::Builder::system()?
        .auth_mechanism(zbus::AuthMechanism::External)
        .build()
        .await
        .context("Failed to connect to session dbus")?;

    tracing::info!("Connecting to portabled");
    let portabled = bus::PortabledDProxy::new(&connection)
        .await
        .context("Failed to connect to dbus service")?;

    match opt.command {
        Command::List => {
            tracing::info!("Listing images");
            for image in portabled
                .list_images()
                .await
                .context("Failed to list portabled images")?
            {
                tracing::info!("{image:#?}");
            }
        }
        Command::Attach { image, profile } => {
            let image = if image.contains('/') {
                let path = Path::new(&image);
                let path = path.canonicalize().context("Failed to canonicalize path")?;
                path.to_str()
                    .context("Path was not valid utf-8")?
                    .to_string()
            } else {
                image
            };

            let unit_file_matches = &[];
            let runtime = true;
            let changes = portabled
                .attach_image(
                    &image,
                    unit_file_matches,
                    &profile,
                    runtime,
                    "", // TODO
                )
                .await
                .context("Failed to attach portable service image")?;

            log_changes(&changes);
        }
        Command::Detach { image } => {
            let image = if image.contains('/') {
                let path = Path::new(&image);
                let path = path.canonicalize().context("Failed to canonicalize path")?;
                path.to_str()
                    .context("Path was not valid utf-8")?
                    .to_string()
            } else {
                image
            };

            let runtime = true;
            let changes = portabled
                .detach_image(&image, runtime)
                .await
                .context("Failed to detach portable service")?;
            log_changes(&changes);
        }
    }

    Ok(())
}

fn log_changes(changes: &[Change]) {
    for change in changes {
        if change.source.is_empty() {
            tracing::info!("- {} {:?}", change.change_type, change.path);
        } else {
            tracing::info!(
                "- {} {:?} -> {:?}",
                change.change_type,
                change.source,
                change.path
            );
        }
    }
}
