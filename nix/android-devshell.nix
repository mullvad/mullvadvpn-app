{ pkgs, android-toolchain }:
pkgs.devshell.mkShell {
  name = "mullvad-android-devshell";
  inherit (android-toolchain) packages;

  env = import ./android-env.nix {
    inherit pkgs;
    inherit (android-toolchain)
      android-sdk
      buildToolsVersion
      ndkVersion
      minSdkVersion
      ;
  };

  devshell.startup.prepare.text = ''
    export FLAKE_ROOT=$(git rev-parse --show-toplevel)
    export ANDROID_ROOT="$FLAKE_ROOT/android"
    cd "$ANDROID_ROOT"
  '';

  # Unfortunately rich menus with package, description and category
  # is only supported using the TOML format and not when using mkShell.
  # The two cannot be combined and TOML format by itself doesn't support
  # the way we dynamically configure the devshell.
  commands = [
    {
      name = "tasks";
      command = "$ANDROID_ROOT/gradlew -p $ANDROID_ROOT tasks";
    }
    {
      name = "build";
      command = "$ANDROID_ROOT/gradlew -p $ANDROID_ROOT assembleOssProdDebug";
    }
    {
      name = "buildFdroid";
      command = "$ANDROID_ROOT/gradlew -p $ANDROID_ROOT assembleOssProdFdroid";
    }
  ];
}
