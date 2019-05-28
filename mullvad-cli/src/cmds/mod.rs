use crate::Command;
use std::collections::HashMap;

mod account;
pub use self::account::Account;

mod auto_connect;
pub use self::auto_connect::AutoConnect;

mod bridge;
pub use self::bridge::Bridge;

mod status;
pub use self::status::Status;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::Disconnect;

mod block_when_disconnected;
pub use self::block_when_disconnected::BlockWhenDisconnected;

mod relay;
pub use self::relay::Relay;

mod lan;
pub use self::lan::Lan;

mod tunnel;
pub use self::tunnel::Tunnel;

mod version;
pub use self::version::Version;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<dyn Command>> {
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(Account),
        Box::new(AutoConnect),
        Box::new(BlockWhenDisconnected),
        Box::new(Bridge),
        Box::new(Connect),
        Box::new(Disconnect),
        Box::new(Lan),
        Box::new(Relay),
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
