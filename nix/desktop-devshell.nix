{ pkgs, desktop-toolchain }:
pkgs.devshell.mkShell {
  name = "mullvad-desktop-devshell";
  packages = desktop-toolchain.packages ++ [
    pkgs.cargo-insta
    pkgs.cargo-deny
  ];

  env = import ./desktop-env.nix {
    inherit pkgs;
  };

  devshell.startup.prepare.text = ''
    export FLAKE_ROOT=$(git rev-parse --show-toplevel)
    cd "$FLAKE_ROOT"
  '';

  commands = [
    {
      name = "build";
      command = "cargo build --manifest-path $FLAKE_ROOT/Cargo.toml";
    }
  ];
}
