use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

pub const MOCK_OPENVPN_ARGS_FILE: &str = "mock_openvpn_args";

pub fn search_openvpn_args<P: AsRef<Path>>(
    openvpn_args_file_path: P,
    search_item: &'static str,
) -> impl Iterator<Item = io::Result<String>> {
    let args_file_path = openvpn_args_file_path.as_ref();
    let args_file = File::open(&args_file_path).expect(&format!(
        "Failed to open mock OpenVPN arguments file: {}",
        args_file_path.display(),
    ));

    let args = BufReader::new(args_file).lines();

    args.skip_while(move |element| {
        element.is_ok() && !element.as_ref().unwrap().contains(search_item)
    })
}
