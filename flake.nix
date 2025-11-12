{
  description = "Mullvad VPN development environments";

  inputs = {
    # Unstable is currently needed for protoc-gen-grpc-java.
    # We should switch to a stable channel once it's avaiable on those.
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
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

      common = import ./nix/common.nix {
        inherit pkgs rust-overlay;
      };

      rust-toolchain = common.rust-toolchain-base.override {
        extensions = ["rust-analyzer"];
        targets = [
          "aarch64-linux-android"
          "armv7-linux-androideabi"
          "x86_64-linux-android"
          "i686-linux-android"
        ];
      };

      desktop-rust-toolchain = common.rust-toolchain-base.override {
        extensions = ["rust-analyzer"];
      };

      versions =
        (builtins.fromTOML (
          builtins.concatStringsSep "\n" (
            builtins.filter (line: !(builtins.match "^[[:space:]]*#" line != null))
            (nixpkgs.lib.splitString "\n" (builtins.readFile ./android/gradle/libs.versions.toml))
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
      packages = {
        inherit android-sdk;
      };

      devShells = {
        android = pkgs.devshell.mkShell {
          name = "mullvad-android-devshell";
          packages = common.commonPackages ++ [
            android-sdk
            rust-toolchain
            pkgs.protoc-gen-grpc-java
            pkgs.jdk17
          ];

          env = import ./nix/android-env.nix {
            inherit pkgs android-sdk buildToolsVersion ndkVersion minSdkVersion;
          };
          # Unfortunately rich menus with package, description and category
          # is only supported using the TOML format and not when using mkShell.
          # The two cannot be combined and TOML format by itself doesn't support
          # the way we dynamically configure the devshell.
          commands = [
            {
              name = "tasks";
              command = "cd android && ./gradlew tasks";
            }
            {
              name = "build";
              command = "cd android && ./gradlew assembleOssProdDebug";
            }
          ];
        };

        desktop = pkgs.devshell.mkShell {
          name = "mullvad-desktop-devshell";
          packages = common.commonPackages ++ [
            desktop-rust-toolchain
          ];

          env = import ./nix/desktop-env.nix {
            inherit pkgs;
          };

          commands = [
            {
              name = "build";
              command = "cargo build";
            }
          ];
        };

        default = self.devShells.${system}.desktop;
      };
    });
}
