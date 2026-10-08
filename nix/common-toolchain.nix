{
  pkgs,
  unstable-pkgs,
}:
let
  rust-toolchain-base = pkgs.buildPackages.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
in
{
  inherit rust-toolchain-base;

  commonPackages = [
    pkgs.git
    pkgs.gcc
    pkgs.gnumake
    pkgs.protobuf
    pkgs.jq
    # The cyclonedx packages are not pinned to the same versions as our containers to avoid complexity
    # and simplify maintenance. If it becomes an issue we should introduce exact version pinning.
    unstable-pkgs.cargo-cyclonedx
    unstable-pkgs.cyclonedx-cli
  ];
}
