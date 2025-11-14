{
  pkgs,
  android,
}:
pkgs.devshell.mkShell {
  name = "mullvad-android-devshell";
  packages = android.packages;

  env = import ./android-env.nix {
    inherit pkgs;
    inherit (android)
      android-sdk
      buildToolsVersion
      ndkVersion
      minSdkVersion
      ;
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
    {
      name = "buildFdroid";
      command = "cd android && ./gradlew assembleOssProdFdroid";
    }
  ];
}
