{
  description = "Mullvad Android app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    android-nixpkgs = {
      url = "github:tadfisher/android-nixpkgs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs-with-grpc-java.url = "github:NixOS/nixpkgs/pull/382872/head";
  };

  outputs = { self, nixpkgs, android-nixpkgs, rust-overlay, flake-utils, nixpkgs-with-grpc-java }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Import the PR's nixpkgs to access the gRPC Java plugin
        prPkgs = import nixpkgs-with-grpc-java { inherit system; };

        # Create an overlay that adds the gRPC Java plugin with the correct name
        grpc-java-overlay = final: prev: {
          protoc-gen-grpc-java = prPkgs.protoc-gen-grpc-java-bin;
        };

        pkgs = import nixpkgs {
          overlays = [ (import rust-overlay) grpc-java-overlay ];
          inherit system;
        };

        minSdkVersion = "26";

        targetArchitectures = [
          "aarch64-linux-android"
          "armv7-linux-androideabi"
          "x86_64-linux-android"
          "i686-linux-android"
        ];

        android-sdk = android-nixpkgs.sdk.${system} (sdkPkgs:
          with sdkPkgs; [
            build-tools-35-0-0
            platforms-android-35
            ndk-27-2-12479018
            cmdline-tools-latest
            platform-tools
          ]);

        rust-toolchain = (pkgs.buildPackages.rust-bin.fromRustupToolchainFile
          ../rust-toolchain.toml).override {
            extensions = [ "rust-analyzer" ];
            targets = targetArchitectures;
          };
      in {
        overlay = final: prev: {
          inherit (self.packages.${system}) android-sdk;
        };

        packages = {
          inherit android-sdk;
          inherit (pkgs) protoc-gen-grpc-java;
        };

        devShells = {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              android-sdk
              rust-toolchain
              cargo
              protobuf
              jdk17
              python3Full
              go
              gcc
              protoc-gen-grpc-java
              gnumake
            ];

            shellHook = ''
              export TEST="hello there"
              export JAVA_HOME="${pkgs.jdk17}"
              export ANDROID_SDK_ROOT="${android-sdk}/share/android-sdk"
              export ANDROID_NDK_ROOT="${android-sdk}/share/android-sdk/ndk/27.2-12479018"
              export NDK_TOOLCHAIN_DIR="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin"

              # Make protoc-gen-grpc-java available in PATH
              export PATH="${pkgs.protoc-gen-grpc-java}/bin:$PATH"

              export AR_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
              export CC_aarch64_linux_android="$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang"
              export CARGO_TARGET_aarch64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang"

              export AR_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/llvm-ar"
              export CC_armv7_linux_androideabi="$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang"
              export CARGO_TARGET_armv7_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang"

              export AR_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
              export CC_x86_64_linux_android="$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang"
              export CARGO_TARGET_x86_64_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang"

              export AR_i686_linux_android="$NDK_TOOLCHAIN_DIR/llvm-ar"
              export CC_i686_linux_android="$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang"
              export CARGO_TARGET_i686_LINUX_ANDROID_LINKER="$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang"

              # Set PROTOC_GEN_GRPC_JAVA_PLUGIN for build scripts that need it
              export PROTOC_GEN_GRPC_JAVA_PLUGIN="${pkgs.protoc-gen-grpc-java}/bin/protoc-gen-grpc-java"

              echo "Mullvad Android development environment loaded!"
            '';
          };
        };
      });
}
