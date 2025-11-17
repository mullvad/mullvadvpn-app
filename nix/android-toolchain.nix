{
  pkgs,
  nixpkgs,
  android-nixpkgs,
  system,
  common-toolchain,
}:
let
  versions =
    (builtins.fromTOML (
      builtins.concatStringsSep "\n" (
        builtins.filter (line: !(builtins.match "^[[:space:]]*#" line != null)) (
          nixpkgs.lib.splitString "\n" (builtins.readFile ../android/gradle/libs.versions.toml)
        )
      )
    )).versions;

  compileSdkVersion = versions."compile-sdk";
  buildToolsVersion = versions."build-tools";
  minSdkVersion = versions."min-sdk";
  ndkVersion = versions.ndk;

  android-sdk = android-nixpkgs.sdk.${system} (
    sdkPkgs: with sdkPkgs; [
      (builtins.getAttr "platforms-android-${compileSdkVersion}" sdkPkgs)
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
    buildToolsVersion
    ndkVersion
    minSdkVersion
    ;

  packages = common-toolchain.commonPackages ++ [
    android-sdk
    rust-toolchain
    pkgs.protoc-gen-grpc-java
    pkgs.jdk17
    pkgs.python314
  ]
  ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
}
