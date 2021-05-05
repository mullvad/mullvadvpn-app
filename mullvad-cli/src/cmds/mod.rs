use crate::Command;
use std::collections::HashMap;

mod account;
pub use self::account::Account;

mod auto_connect;
pub use self::auto_connect::AutoConnect;

mod beta_program;
pub use self::beta_program::BetaProgram;

mod block_when_disconnected;
pub use self::block_when_disconnected::BlockWhenDisconnected;

mod bridge;
pub use self::bridge::Bridge;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::Disconnect;

mod dns;
pub use self::dns::Dns;

mod lan;
pub use self::lan::Lan;

mod custom_dns;
pub use self::custom_dns::CustomDns;

mod reconnect;
pub use self::reconnect::Reconnect;

mod relay;
pub use self::relay::Relay;

mod reset;
pub use self::reset::Reset;

#[cfg(target_os = "linux")]
mod split_tunnel;
#[cfg(target_os = "linux")]
pub use self::split_tunnel::SplitTunnel;

mod status;
pub use self::status::Status;

mod tunnel;
pub use self::tunnel::Tunnel;

mod version;
pub use self::version::Version;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<dyn Command>> {
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(Account),
        Box::new(AutoConnect),
        Box::new(BetaProgram),
        Box::new(BlockWhenDisconnected),
        Box::new(Bridge),
        Box::new(Connect),
        Box::new(Disconnect),
        Box::new(Dns),
        Box::new(Reconnect),
        Box::new(Lan),
        #[cfg(not(target_os = "android"))]
        Box::new(CustomDns),
        Box::new(Relay),
        Box::new(Reset),
        #[cfg(target_os = "linux")]
        Box::new(SplitTunnel),
        Box::new(Status),
        Box::new(Tunnel),
        Box::new(Version),
    ];
    let mut map = HashMap::new();
    for cmd in commands {
        if map.insert(cmd.name(), cmd).is_some() {
            panic!("Multiple commands with the same name");
        }
    }
    map
}
