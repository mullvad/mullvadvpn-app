use std::{env::args, net::SocketAddr};
use tunnel_obfuscation::{create_obfuscator, Obfuscator, Settings, Udp2TcpSettings};

#[tokio::main]
async fn main() {
    if args().len() != 2 {
        println!("Missing arguments");
    }

    let obfuscator = instantiate_requested(&args().last().unwrap()).await;

    println!("endpoint() returns {:?}", obfuscator.endpoint());

    if let Err(err) = obfuscator.run().await {
        println!("obfuscator.run() failed: {err:?}");
    }
}

async fn instantiate_requested(obfuscator_type: &str) -> Box<dyn Obfuscator> {
    match obfuscator_type {
        "udp2tcp" => {
            let settings = Udp2TcpSettings {
                peer: SocketAddr::new("127.0.0.1".parse().unwrap(), 3030),
                #[cfg(target_os = "linux")]
                fwmark: Some(1337),
            };

            create_obfuscator(&Settings::Udp2Tcp(settings))
                .await
                .expect("Creating obfuscator failed")
        }
        _ => {
            unimplemented!()
        }
    }
}
