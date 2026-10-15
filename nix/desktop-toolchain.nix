{
  pkgs,
  common-toolchain,
}:
let
  desktop-rust-toolchain = common-toolchain.rust-toolchain-base.override {
    extensions = [ "rust-analyzer" ];
  };
in
{
  inherit desktop-rust-toolchain;

  packages =
    common-toolchain.commonPackages
    ++ [
      desktop-rust-toolchain
      pkgs.stdenv.cc
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
      pkgs.pkg-config
      pkgs.dbus.dev
    ];
}
