{
  pkgs,
  desktop,
}:
pkgs.devshell.mkShell {
  name = "mullvad-desktop-devshell";
  packages = desktop.packages;

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
