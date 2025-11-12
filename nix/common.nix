{
  pkgs,
  rust-overlay,
}:
let
  rust-toolchain-base = pkgs.buildPackages.rust-bin.fromRustupToolchainFile
    ../rust-toolchain.toml;
in
{
  inherit rust-toolchain-base;

  commonPackages = [
    pkgs.gcc
    pkgs.gnumake
    pkgs.protobuf
    pkgs.python314
    pkgs.pkg-config
  ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
    pkgs.dbus.dev
  ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.libiconv
  ];
}
