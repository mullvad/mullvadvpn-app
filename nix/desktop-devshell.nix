{
  pkgs,
  desktop-toolchain,
}:
pkgs.devshell.mkShell {
  name = "mullvad-desktop-devshell";
  packages = desktop-toolchain.packages;

  env = import ./desktop-env.nix {
    inherit pkgs;
  };

  commands = [
    {
      name = "build";
      command = "cargo build";
    }
  ];
}
