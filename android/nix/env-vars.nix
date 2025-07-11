{
  pkgs,
  android-sdk,
  buildToolsVersion,
  ndkVersion,
  minSdkVersion,
}: [
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
    value = "${android-sdk}/share/android-sdk/ndk/${ndkVersion}/toolchains/llvm/prebuilt/linux-x86_64/bin";
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
