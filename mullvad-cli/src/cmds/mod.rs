use std::collections::HashMap;
use Command;

mod account;
pub use self::account::Account;

mod auto_connect;
pub use self::auto_connect::AutoConnect;

mod status;
pub use self::status::Status;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::Disconnect;

mod relay;
pub use self::relay::Relay;

mod lan;
pub use self::lan::Lan;

mod tunnel;
pub use self::tunnel::Tunnel;

mod version;
pub use self::version::Version;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<Command>> {
    let commands: Vec<Box<Command>> = vec![
        Box::new(Account),
        Box::new(AutoConnect),
        Box::new(Status),
        Box::new(Connect),
        Box::new(Disconnect),
        Box::new(Relay),
        Box::new(Lan),
        Box::new(Tunnel),
        Box::new(Version),
    ];
    let mut map = HashMap::new();
    for cmd in commands {
        if let Some(_) = map.insert(cmd.name(), cmd) {
            panic!("Multiple commands with the same name");
        }
    }
    map
}
