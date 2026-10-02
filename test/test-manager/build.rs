use std::env::var;

fn main() {
    // Rebuild if SSH provision scripts change
    println!("cargo::rerun-if-changed=../scripts/ssh-setup.sh");
    println!("cargo::rerun-if-changed=../scripts/ssh-setup.ps1");

    let link_statically = var("TEST_MANAGER_STATIC").is_ok_and(|x| x != "0");

    if link_statically {
        println!("cargo::rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
        println!("cargo::rustc-link-lib=static=pcap");
    }
}
