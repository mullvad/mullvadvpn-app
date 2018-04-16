#[cfg(windows)]
extern crate simple_service;

#[cfg(windows)]
fn main() {
    simple_service::run();
}

#[cfg(not(windows))]
fn main() {
    panic!("This program is only intended to run on Windows.");
}