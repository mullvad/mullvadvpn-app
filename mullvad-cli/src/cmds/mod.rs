use Command;
use std::collections::HashMap;

mod account;
pub use self::account::Account;

mod status;
pub use self::status::Status;

mod connect;
pub use self::connect::Connect;

mod disconnect;
pub use self::disconnect::Disconnect;

mod custom_relay;
pub use self::custom_relay::CustomRelay;

mod shutdown;
pub use self::shutdown::Shutdown;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<Command>> {
    let commands: Vec<Box<Command>> = vec![
        Box::new(Account),
        Box::new(Status),
        Box::new(Connect),
        Box::new(Disconnect),
        Box::new(CustomRelay),
        Box::new(Shutdown),
    ];
    let mut map = HashMap::new();
    for cmd in commands {
        if let Some(_) = map.insert(cmd.name(), cmd) {
            panic!("Multiple commands with the same name");
        }
    }
    map
}
