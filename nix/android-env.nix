{
  pkgs,
  android-sdk,
  buildToolsVersion,
  ndkVersion,
  minSdkVersion,
}: let
  hostPlatform =
    # For linux the NDK support is limited to x86_64.
    if pkgs.stdenv.isLinux && pkgs.stdenv.isx86_64
    then "linux-x86_64"
    # For macOS the x86_64 NDK is used for both intel and arm (via rosetta).
    else if pkgs.stdenv.isDarwin
    then "darwin-x86_64"
    else throw "Unsupported OS/architecture combination: ${pkgs.stdenv.hostPlatform.system}";
in
  [
    {
      name = "JAVA_HOME";
      value = "${pkgs.jdk17}";
    }
    {
      name = "PROTOC_GEN_GRPC_JAVA_PLUGIN";
      prefix = "${pkgs.protoc-gen-grpc-java}/bin/protoc-gen-grpc-java";
    }
    {
      name = "GRADLE_OPTS";
      value = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${android-sdk}/share/android-sdk/build-tools/${buildToolsVersion}/aapt2";
    }
    {
      name = "ANDROID_HOME";
      value = "${android-sdk}/share/android-sdk";
    }
    {
      name = "ANDROID_SDK_ROOT";
      value = "${android-sdk}/share/android-sdk";
    }
    {
      name = "ANDROID_NDK_ROOT";
      value = "${android-sdk}/share/android-sdk/ndk/${ndkVersion}";
    }
    {
      name = "NDK_TOOLCHAIN_DIR";
      value = "${android-sdk}/share/android-sdk/ndk/${ndkVersion}/toolchains/llvm/prebuilt/${hostPlatform}/bin";
    }
    {
      name = "AR_aarch64_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
    }
    {
      name = "CC_aarch64_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang";
    }
    {
      name = "CARGO_TARGET_aarch64_LINUX_ANDROID_LINKER";
      value = "$NDK_TOOLCHAIN_DIR/aarch64-linux-android${minSdkVersion}-clang";
    }
    {
      name = "AR_armv7_linux_androideabi";
      value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
    }
    {
      name = "CC_armv7_linux_androideabi";
      value = "$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang";
    }
    {
      name = "CARGO_TARGET_armv7_LINUX_ANDROID_LINKER";
      value = "$NDK_TOOLCHAIN_DIR/armv7-linux-androideabi${minSdkVersion}-clang";
    }
    {
      name = "AR_x86_64_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
    }
    {
      name = "CC_x86_64_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang";
    }
    {
      name = "CARGO_TARGET_x86_64_LINUX_ANDROID_LINKER";
      value = "$NDK_TOOLCHAIN_DIR/x86_64-linux-android${minSdkVersion}-clang";
    }
    {
      name = "AR_i686_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/llvm-ar";
    }
    {
      name = "CC_i686_linux_android";
      value = "$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang";
    }
    {
      name = "CARGO_TARGET_i686_LINUX_ANDROID_LINKER";
      value = "$NDK_TOOLCHAIN_DIR/i686-linux-android${minSdkVersion}-clang";
    }
  ]
  ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
    {
      name = "LIBRARY_PATH";
      value = "${pkgs.libiconv}/lib";
    }
    {
      name = "CPATH";
      value = "${pkgs.libiconv}/include";
    }
    {
      name = "RUSTFLAGS";
      value = "-L${pkgs.libiconv}/lib";
    }
  ]
