{ pkgs }:
[
  {
    name = "PKG_CONFIG_PATH";
    value = "${pkgs.dbus.dev}/lib/pkgconfig";
  }
  {
    name = "LD_LIBRARY_PATH";
    value = pkgs.lib.makeLibraryPath (
      with pkgs;
      [
        # Electron
        glib
        nspr
        nss
        atk
        cups
        dbus
        cairo
        gtk3
        pango
        libx11
        libxcomposite
        libxdamage
        libxext
        libxfixes
        libxrandr
        libgbm
        expat
        libxcb
        libxkbcommon
        alsa-lib
        libglvnd
      ]
    );
  }
]
