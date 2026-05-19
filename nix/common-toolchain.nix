{ pkgs }:
let
  rust-toolchain-base = pkgs.buildPackages.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
in
{
  inherit rust-toolchain-base;

  commonPackages = [
    pkgs.git
    pkgs.gcc
    pkgs.gnumake
  ];
}
