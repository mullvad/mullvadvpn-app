use Command;
use std::collections::HashMap;

mod account;
pub use self::account::*;

/// Returns a map of all available subcommands with their name as key.
pub fn get_commands() -> HashMap<&'static str, Box<Command>> {
    let commands = vec![Box::new(Account) as Box<Command>];
    let mut map = HashMap::new();
    for cmd in commands {
        if let Some(_) = map.insert(cmd.name(), cmd) {
            panic!("Multiple commands with the same name");
        }
    }
    map
}
