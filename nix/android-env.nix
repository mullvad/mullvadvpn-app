{
  pkgs,
  android-sdk,
  jdk,
  buildToolsVersion,
  ndkVersion,
  minSdkVersion,
}:
let
  hostPlatform =
    # For linux the NDK support is limited to x86_64.
    if pkgs.stdenv.isLinux && pkgs.stdenv.isx86_64 then
      "linux-x86_64"
    # For macOS the x86_64 NDK is used for both intel and arm (via rosetta).
    else if pkgs.stdenv.isDarwin then
      "darwin-x86_64"
    else
      throw "Unsupported OS/architecture combination: ${pkgs.stdenv.hostPlatform.system}";
  ndkToolchainDir = "${android-sdk}/share/android-sdk/ndk/${ndkVersion}/toolchains/llvm/prebuilt/${hostPlatform}/bin";
in
[
  {
    name = "JAVA_HOME";
    value = "${jdk}";
  }
  {
    name = "GRADLE_OPTS";
    value = builtins.concatStringsSep " " [
      "-Dorg.gradle.project.android.aapt2FromMavenOverride=${android-sdk}/share/android-sdk/build-tools/${buildToolsVersion}/aapt2"
      "-Dorg.gradle.project.android.sync.suppressAgpWarnings=UNSUPPORTED_PROJECT_OPTION_USE"
    ];
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
    value = ndkToolchainDir;
  }
  {
    name = "AR_aarch64_linux_android";
    value = "${ndkToolchainDir}/llvm-ar";
  }
  {
    name = "CC_aarch64_linux_android";
    value = "${ndkToolchainDir}/aarch64-linux-android${minSdkVersion}-clang";
  }
  {
    name = "CARGO_TARGET_aarch64_LINUX_ANDROID_LINKER";
    value = "${ndkToolchainDir}/aarch64-linux-android${minSdkVersion}-clang";
  }
  {
    name = "AR_armv7_linux_androideabi";
    value = "${ndkToolchainDir}/llvm-ar";
  }
  {
    name = "CC_armv7_linux_androideabi";
    value = "${ndkToolchainDir}/armv7-linux-androideabi${minSdkVersion}-clang";
  }
  {
    name = "CARGO_TARGET_armv7_LINUX_ANDROID_LINKER";
    value = "${ndkToolchainDir}/armv7-linux-androideabi${minSdkVersion}-clang";
  }
  {
    name = "AR_x86_64_linux_android";
    value = "${ndkToolchainDir}/llvm-ar";
  }
  {
    name = "CC_x86_64_linux_android";
    value = "${ndkToolchainDir}/x86_64-linux-android${minSdkVersion}-clang";
  }
  {
    name = "CARGO_TARGET_x86_64_LINUX_ANDROID_LINKER";
    value = "${ndkToolchainDir}/x86_64-linux-android${minSdkVersion}-clang";
  }
  {
    name = "AR_i686_linux_android";
    value = "${ndkToolchainDir}/llvm-ar";
  }
  {
    name = "CC_i686_linux_android";
    value = "${ndkToolchainDir}/i686-linux-android${minSdkVersion}-clang";
  }
  {
    name = "CARGO_TARGET_i686_LINUX_ANDROID_LINKER";
    value = "${ndkToolchainDir}/i686-linux-android${minSdkVersion}-clang";
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
