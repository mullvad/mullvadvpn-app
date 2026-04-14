{
  pkgs,
  nixpkgs,
  android-nixpkgs,
  common-toolchain,
}:
let
  inherit
    (builtins.fromTOML (
      builtins.concatStringsSep "\n" (
        builtins.filter (line: !(builtins.match "^[[:space:]]*#" line != null)) (
          nixpkgs.lib.splitString "\n" (builtins.readFile ../android/gradle/libs.versions.toml)
        )
      )
    ))
    versions
    ;

  compileSdkVersion = versions."compile-sdk-major";
  compileSdkMinorVersion = versions."compile-sdk-minor" or "0";
  buildToolsVersion = versions."build-tools";
  minSdkVersion = versions."min-sdk";
  ndkVersion = versions.ndk;
  jdk = pkgs."jdk${versions."jvm-toolchain"}";

  android-sdk = (import "${android-nixpkgs}" { pkgs = pkgs // { openjdk = jdk; }; }).sdk (
    sdkPkgs: with sdkPkgs; [
      (builtins.getAttr "platforms-android-${compileSdkVersion}-${compileSdkMinorVersion}" sdkPkgs)
      (builtins.getAttr "build-tools-${builtins.replaceStrings [ "." ] [ "-" ] buildToolsVersion}" sdkPkgs)
      (builtins.getAttr "ndk-${builtins.replaceStrings [ "." ] [ "-" ] ndkVersion}" sdkPkgs)
      cmdline-tools-latest
      platform-tools
    ]
  );

  rust-toolchain = common-toolchain.rust-toolchain-base.override {
    extensions = [ "rust-analyzer" ];
    targets = [
      "aarch64-linux-android"
      "armv7-linux-androideabi"
      "x86_64-linux-android"
      "i686-linux-android"
    ];
  };
in
{
  inherit
    android-sdk
    rust-toolchain
    jdk
    buildToolsVersion
    ndkVersion
    minSdkVersion
    ;

  packages =
    common-toolchain.commonPackages
    ++ [
      android-sdk
      rust-toolchain
      pkgs.protoc-gen-grpc-java
      jdk
      pkgs.python314
    ]
    ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
}
