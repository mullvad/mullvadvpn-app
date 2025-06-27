{
  description = "Mullvad Android app build flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
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
    grpc-nixpkgs-pr.url = "github:NixOS/nixpkgs/pull/382872/head";
  };

  outputs = { self, nixpkgs, android-nixpkgs, rust-overlay, flake-utils
    , grpc-nixpkgs-pr, devshell }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        grpc-nixpkgs-pr-overlay = final: prev: {
          protoc-gen-grpc-java = (import grpc-nixpkgs-pr {
            inherit system;
          }).protoc-gen-grpc-java-bin;
        };

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
            grpc-nixpkgs-pr-overlay
            devshell.overlays.default
          ];
        };

        targetArchitectures = [
          "aarch64-linux-android"
          "armv7-linux-androideabi"
          "x86_64-linux-android"
          "i686-linux-android"
        ];

        # TODO: Extract from versions.toml.
        compileSdkVersion = "36";
        buildToolsVersion = "36-0-0";
        androidVersion = "36.0.0";
        ndkVersion = "27-2-12479018";
        ndkVersion2 = "27.2.12479018";
        minSdkVersion = "26";

        android-sdk = android-nixpkgs.sdk.${system} (sdkPkgs:
          with sdkPkgs; [
            (builtins.getAttr "platforms-android-${compileSdkVersion}" sdkPkgs)
            (builtins.getAttr "build-tools-${buildToolsVersion}" sdkPkgs)
            (builtins.getAttr "ndk-${ndkVersion}" sdkPkgs)
            cmdline-tools-latest
            platform-tools
          ]);

        rust-toolchain = (pkgs.buildPackages.rust-bin.fromRustupToolchainFile
          ../rust-toolchain.toml).override {
            extensions = [ "rust-analyzer" ];
            targets = targetArchitectures;
          };
        go_1_24_2 = import (fetchTarball {
          url = "https://github.com/NixOS/nixpkgs/archive/1f426f65ac4e6bf808923eb6f8b8c2bfba3d18c5.tar.gz";
          sha256 = "0xi47nilddmwnq0wxsg7wsvpqz9xq12mvn11a9kqr9qbdpwhsh71";
        } ) { inherit system; };

      in {
        overlay = final: prev: {
          inherit (self.packages.${system}) android-sdk;
        };

        packages = {
          inherit android-sdk;
          inherit (pkgs) protoc-gen-grpc-java;
        };

        devShells.default = pkgs.devshell.mkShell {
          name = "mullvad-android-devshell";
          packages = [
            android-sdk
            rust-toolchain
            go_1_24_2.go
            pkgs.protoc-gen-grpc-java
            pkgs.gcc
            pkgs.gnumake
            pkgs.protobuf
            pkgs.jdk17
            pkgs.python3Full
          ];
          # TODO: Cleanup! Generate arch vars?
          env = [
            {
              name = "PATH";
              prefix = "${pkgs.protoc-gen-grpc-java}/bin";
            }
            {
              name = "PROTOC_GEN_GRPC_JAVA_PLUGIN";
              prefix = "${pkgs.protoc-gen-grpc-java}/bin/protoc-gen-grpc-java";
            }
            {
              name = "JAVA_HOME";
              value = "${pkgs.jdk17}";
            }
            {
              name = "GRADLE_OPTS";
              value =
                "-Dorg.gradle.project.android.aapt2FromMavenOverride=${android-sdk}/share/android-sdk/build-tools/${androidVersion}/aapt2";
            }
            {
              name = "ANDROID_HOME";
              value = "${android-sdk}/share/android-sdk";
            }
            # {
            #   name = "ANDROID_SDK_ROOT";
            #   value = "${android-sdk}/share/android-sdk";
            # }
            {
              name = "ANDROID_NDK_ROOT";
              value = "${android-sdk}/share/android-sdk/ndk/${ndkVersion2}";
            }
            {
              name = "NDK_TOOLCHAIN_DIR";
              value =
                "${android-sdk}/share/android-sdk/ndk/${ndkVersion2}/toolchains/llvm/prebuilt/linux-x86_64/bin";
            }
            {
              name = "AR_aarch64_linux_android";
              value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
            }
            {
              name = "CC_aarch64_linux_android";
              value =
                "$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang";
            }
            {
              name = "CARGO_TARGET_aarch64_LINUX_ANDROID_LINKER";
              value =
                "$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang";
            }
            {
              name = "AR_armv7_linux_androideabi";
              value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
            }
            {
              name = "CC_armv7_linux_androideabi";
              value =
                "$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang";
            }
            {
              name = "CARGO_TARGET_armv7_LINUX_ANDROID_LINKER";
              value =
                "$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang";
            }
            {
              name = "AR_x86_64_linux_android";
              value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
            }
            {
              name = "CC_x86_64_linux_android";
              value =
                "$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang";
            }
            {
              name = "CARGO_TARGET_x86_64_LINUX_ANDROID_LINKER";
              value =
                "$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang";
            }
            {
              name = "AR_i686_linux_android";
              value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
            }
            {
              name = "CC_i686_linux_android";
              value =
                "$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang";
            }
            {
              name = "CARGO_TARGET_i686_LINUX_ANDROID_LINKER";
              value =
                "$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang";
            }
          ];
        };
      });
}
