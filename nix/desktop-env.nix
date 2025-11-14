{ pkgs }:
[
  {
    name = "PKG_CONFIG_PATH";
    value = "${pkgs.dbus.dev}/lib/pkgconfig";
  }
]
