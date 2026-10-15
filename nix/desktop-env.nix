{ pkgs }:
pkgs.lib.optionals pkgs.stdenv.isLinux [
  {
    name = "PKG_CONFIG_PATH";
    value = "${pkgs.dbus.dev}/lib/pkgconfig";
  }
]
++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
  {
    name = "LIBRARY_PATH";
    value = pkgs.lib.makeLibraryPath [
      pkgs.libiconv
      pkgs.darwin.libpcap
    ];
  }
  {
    name = "SDKROOT";
    value = pkgs.apple-sdk.sdkroot;
  }
  {
    name = "MACOSX_DEPLOYMENT_TARGET";
    value = pkgs.stdenv.hostPlatform.darwinMinVersion;
  }
]
