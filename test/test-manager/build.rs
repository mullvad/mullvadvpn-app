fn main() {
    // Rebuild if SSH provision script changes
    println!("cargo::rerun-if-changed=../scripts/ssh-setup.sh");

    println!("cargo::rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
    println!("cargo::rustc-link-lib=static=pcap");
}
