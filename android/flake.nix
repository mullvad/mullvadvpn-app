{
  description = "Mullvad Android app build flake";

  inputs = {
    # Unstable is currently needed for protoc-gen-grpc-java.
    # We should switch to a stable channel once it's avaiable on those.
    nixpkgs.url = "nixpkgs/nixos-unstable";
    devshell.url = "github:numtide/devshell";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    android-nixpkgs,
    rust-overlay,
    flake-utils,
    devshell,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          devshell.overlays.default
        ];
      };

      rust-toolchain = (pkgs.buildPackages.rust-bin.fromRustupToolchainFile
        ../rust-toolchain.toml).override {
        extensions = ["rust-analyzer"];
        targets = [
          "aarch64-linux-android"
          "armv7-linux-androideabi"
          "x86_64-linux-android"
          "i686-linux-android"
        ];
      };

      patchedGo_1_21_3 =
        (import (fetchTarball {
          url = "https://github.com/NixOS/nixpkgs/archive/b392079f5fd051926a834c878d27ceec4f139dce.tar.gz";
          sha256 = "16dkk98fs9pw2amz0vpjsc7ks85cw3hc5rlpbp27llq6x7lwpjaz";
        }) {inherit system;}).go_1_21.overrideAttrs (oldAttrs: {
          patches = (oldAttrs.patches or []) ++ [./docker/goruntime-boottime-over-monotonic.diff];
        });

      versions =
        (builtins.fromTOML (
          builtins.concatStringsSep "\n" (
            builtins.filter (line: !(builtins.match "^[[:space:]]*#" line != null))
            (nixpkgs.lib.splitString "\n" (builtins.readFile ./gradle/libs.versions.toml))
          )
        )).versions;

      compileSdkVersion = versions."compile-sdk";
      buildToolsVersion = versions."build-tools";
      minSdkVersion = versions."min-sdk";
      ndkVersion = versions.ndk;

      android-sdk = android-nixpkgs.sdk.${system} (sdkPkgs:
        with sdkPkgs; [
          (builtins.getAttr "platforms-android-${compileSdkVersion}" sdkPkgs)
          (builtins.getAttr "build-tools-${builtins.replaceStrings ["."] ["-"] buildToolsVersion}" sdkPkgs)
          (builtins.getAttr "ndk-${builtins.replaceStrings ["."] ["-"] ndkVersion}" sdkPkgs)
          cmdline-tools-latest
          platform-tools
        ]);
    in {
      overlay = final: prev: {
        inherit (self.packages.${system}) android-sdk;
      };

      packages = {
        inherit android-sdk;
      };

      devShells.default = pkgs.devshell.mkShell {
        name = "mullvad-android-devshell";
        packages = [
          android-sdk
          rust-toolchain
          patchedGo_1_21_3
          pkgs.protoc-gen-grpc-java
          pkgs.gcc
          pkgs.gnumake
          pkgs.protobuf
          pkgs.jdk17
          pkgs.python3Full
        ];
        env = import ./nix/env-vars.nix {
          inherit pkgs android-sdk buildToolsVersion ndkVersion minSdkVersion;
        };
      };
    });
}
