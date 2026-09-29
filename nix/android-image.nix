{
  pkgs,
  android-toolchain,
  tag ? "latest",
}:
let
  inherit (pkgs) lib;
  inherit (android-toolchain)
    android-sdk
    jdk
    buildToolsVersion
    ndkVersion
    minSdkVersion
    ;

  sdkRoot = "${android-sdk}/share/android-sdk";
  ndkRoot = "${sdkRoot}/ndk/${ndkVersion}";
  aapt2 = "${sdkRoot}/build-tools/${buildToolsVersion}/aapt2";
  caBundle = "/etc/ssl/certs/ca-certificates.crt";

  path = lib.concatStringsSep ":" [
    "${jdk}/bin"
    "${sdkRoot}/build-tools/${buildToolsVersion}"
    "/bin"
    "/usr/bin"
    "/sbin"
  ];

  devshellEnv = import ./android-env.nix {
    inherit
      pkgs
      android-sdk
      jdk
      buildToolsVersion
      ndkVersion
      minSdkVersion
      ;
  };

  etcFiles = pkgs.symlinkJoin {
    name = "android-image-etc";
    paths = lib.mapAttrsToList pkgs.writeTextDir {
      "etc/passwd" = ''
        root:x:0:0:root:/root:/bin/bash
        nobody:x:65534:65534:nobody:/var/empty:/bin/false
      '';
      "etc/group" = ''
        root:x:0:
        nogroup:x:65534:
      '';
      "etc/nsswitch.conf" = ''
        hosts: files dns
      '';
      "usr/local/etc/pkcs11_java.cfg" = ''
        name = YKCS11
        description = SunPKCS11 via YKCS11
        library = ${lib.getLib pkgs.yubico-piv-tool}/lib/libykcs11.so
        slotListIndex = 0
      '';
    };
  };
in
pkgs.dockerTools.buildLayeredImage {
  name = "mullvadvpn-app-build-android-nix";
  inherit tag;

  contents =
    [
      pkgs.bashInteractive
      pkgs.dockerTools.binSh
      pkgs.dockerTools.usrBinEnv
      pkgs.dockerTools.caCertificates
      etcFiles
    ]
    ++ pkgs.stdenv.initialPath
    ++ android-toolchain.packages
    ++ (with pkgs; [
      curl
      fdroidserver
      gnupg
      html-tidy
      less
      nodejs
      pcsclite
      procps
      rclone
      unzip
      which
      yubico-piv-tool
      zip
    ]);

  config = {
    Cmd = [ "/bin/bash" ];
    WorkingDir = "/build";

    Labels = {
      "org.opencontainers.image.source" = "https://github.com/mullvad/mullvadvpn-app";
      "org.opencontainers.image.description" = "Mullvad VPN app Android build container";
      "org.opencontainers.image.licenses" = "GPL-3.0-or-later";
    };

    Env =
      map (entry: "${entry.name}=${entry.value or entry.prefix}") devshellEnv
      ++ [
        "PATH=${path}"
        "HOME=/root"
        "LANG=C.UTF-8"
        "ANDROID_NDK_HOME=${ndkRoot}"
        "GRADLE_USER_HOME=/root/.gradle"
        "ORG_GRADLE_PROJECT_android.aapt2FromMavenOverride=${aapt2}"
        "CARGO_TARGET_DIR=/cargo-target/target"
        "RUST_ANDROID_GRADLE_PYTHON_COMMAND=${pkgs.python314}/bin/python3"
        "SSL_CERT_FILE=${caBundle}"
        "CURL_CA_BUNDLE=${caBundle}"
      ];
  };

  extraCommands = ''
    mkdir -p build cargo-target root/.gradle root/.cargo/registry run/pcscd tmp
    chmod 1777 tmp
  '';
}
