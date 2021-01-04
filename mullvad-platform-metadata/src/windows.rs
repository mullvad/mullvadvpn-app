use std::collections::HashMap;

mod command;
use command::command_stdout_lossy;

pub fn version() -> String {
    let system_info = system_info();
    let os_name = system_info.get("OS Name");
    let os_version = system_info.get("OS Version");
    let version = os_name.map(parse_version).unwrap_or(String::from("N/A"));
    let full_version = os_version
        .map(parse_full_version)
        .unwrap_or(String::from("N/A"));
    format!("Windows {} ({})", version, full_version)
}

pub fn short_version() -> String {
    let system_info = system_info();
    let os_name = system_info.get("OS Name");
    let version = os_name.map(parse_version).unwrap_or(String::from("N/A"));
    format!("Windows {}", version)
}

fn system_info() -> HashMap<String, String> {
    let system_info =
        command_stdout_lossy("systeminfo", &["/FO", "LIST"]).unwrap_or_else(String::new);

    let mut info_map = HashMap::new();
    system_info.lines().for_each(|line| {
        let mut split = line.split(":");
        if let Some(key) = split.next() {
            if let Some(value) = split.next() {
                info_map.insert(key.to_owned(), value.to_owned());
            }
        }
    });

    info_map
}

fn parse_version(os_name: &String) -> String {
    os_name
        .trim()
        .trim_start_matches("Microsoft Windows ")
        .to_owned()
}

fn parse_full_version(os_version: &String) -> String {
    os_version.trim().to_owned()
}

pub fn extra_metadata() -> impl Iterator<Item = (String, String)> {
    std::iter::empty()
}
