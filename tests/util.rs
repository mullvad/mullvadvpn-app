use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn read_file<P: AsRef<Path>>(path: P) -> String {
    let mut string = String::new();
    let mut f = File::open(path).unwrap();
    f.read_to_string(&mut string).unwrap();
    string
}

#[cfg(target_os = "linux")]
pub fn read_args_for_proc(pid: u32) -> Vec<String> {
    let cmdline = read_file(format!("/proc/{}/cmdline", pid));
    cmdline.split_terminator('\0').map(String::from).collect()
}
