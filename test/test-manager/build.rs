fn main() {
    // Rebuild if SSH provision script changes
    println!("cargo:rerun-if-changed=../scripts/ssh-setup.sh");
}
