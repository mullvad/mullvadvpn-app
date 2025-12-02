#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ok(())
}

#[cfg(not(feature = "domain-fronting"))]
pub mod imp {
    pub async fn main() -> anyhow::Result<()> {
        unimplemented!(
            "cargo run -p mullvad-api --features domain-fronting --bin domain_fronting -- --front <FRONT_DOMAIN> --host <HOST_DOMAIN>"
        )
    }
}


