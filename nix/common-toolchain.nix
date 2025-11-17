{ pkgs, rust-overlay }:
let
  rust-toolchain-base = pkgs.buildPackages.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
in
{
  inherit rust-toolchain-base;

  commonPackages = [
    pkgs.gcc
    pkgs.gnumake
    pkgs.protobuf
  ];
}
