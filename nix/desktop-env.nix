{ pkgs }:
[ ]
++ pkgs.lib.optionals pkgs.stdenv.isLinux [
  {
    name = "PKG_CONFIG_PATH";
    value = "${pkgs.dbus.dev}/lib/pkgconfig";
  }
]
++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
  {
    name = "LIBRARY_PATH";
    value = "${pkgs.libiconv}/lib";
  }
  {
    name = "CPATH";
    value = "${pkgs.libiconv}/include";
  }
  {
    name = "RUSTFLAGS";
    value = "-L${pkgs.libiconv}/lib";
  }
]
