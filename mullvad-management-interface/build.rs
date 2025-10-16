fn main() {
    tonic_build::compile_protos("proto/management_interface.proto").unwrap();

    // Enable DAITA by default on desktop and android
    println!("cargo::rustc-check-cfg=cfg(daita)");
    println!(r#"cargo::rustc-cfg=daita"#);
}
