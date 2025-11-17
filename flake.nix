{
  description = "Mullvad VPN development environments";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
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
  };

  outputs =
    {
      self,
      nixpkgs,
      android-nixpkgs,
      rust-overlay,
      flake-utils,
      devshell,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
            devshell.overlays.default
          ];
        };

        android-toolchain = import ./nix/android-toolchain.nix {
          inherit
            pkgs
            nixpkgs
            android-nixpkgs
            system
            rust-overlay
            ;
        };

      in
      {
        devShells.default = import ./nix/android-devshell.nix {
          inherit pkgs self android-toolchain;
        };
      }
    );
}
