use clap::Parser;

#[derive(Parser)]
enum Opt {
    Up {},
    Down {},
}

#[tokio::main]
async fn main() -> nullvad::Result<()> {
    let opt = Opt::parse();

    env_logger::init();

    match opt {
        Opt::Up {} => nullvad::up().await?,
        Opt::Down {} => nullvad::down(),
    }

    Ok(())
}
