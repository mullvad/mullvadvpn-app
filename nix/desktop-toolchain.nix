{
  pkgs,
  common-toolchain,
}:
let
  desktop-rust-toolchain = common-toolchain.rust-toolchain-base.override {
    extensions = [ "rust-analyzer" ];
  };

  package-json = builtins.fromJSON (builtins.readFile ../desktop/package.json);
  node-version = builtins.head (pkgs.lib.splitString "." package-json.volta.node);
in
{
  inherit desktop-rust-toolchain;

  packages = common-toolchain.commonPackages ++ [
    desktop-rust-toolchain
    pkgs."nodejs_${node-version}"
    pkgs.pkg-config
    pkgs.dbus.dev
  ];
}
