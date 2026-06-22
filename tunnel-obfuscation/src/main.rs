use std::{env::args, net::SocketAddr};
use tunnel_obfuscation::{
    LocalSocketObfuscator, Settings, create_local_socket_obfuscator, udp2tcp,
};

#[tokio::main]
async fn main() {
    if args().len() != 2 {
        println!("Missing arguments");
    }

    let obfuscator = instantiate_requested(&args().next_back().unwrap()).await;

    println!("endpoint() returns {:?}", obfuscator.endpoint());

    if let Err(err) = obfuscator.run().await {
        println!("obfuscator.run() failed: {err:?}");
    }
}

async fn instantiate_requested(obfuscator_type: &str) -> Box<dyn LocalSocketObfuscator> {
    match obfuscator_type {
        "udp2tcp" => {
            let settings = udp2tcp::Settings {
                peer: SocketAddr::new("127.0.0.1".parse().unwrap(), 3030),
                #[cfg(target_os = "linux")]
                fwmark: Some(1337),
            };

            create_local_socket_obfuscator(&Settings::Udp2Tcp(settings))
                .await
                .expect("Creating obfuscator failed")
        }
        _ => {
            unimplemented!()
        }
    }
}
