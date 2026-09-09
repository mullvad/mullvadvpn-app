{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-26.05";
    swift-overlay.url = "github:Comamoca/swift-overlay";
  };

  outputs =
    inputs@{ nixpkgs, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems =
        f:
        builtins.listToAttrs (
          map (system: {
            name = system;
            value = f system;
          }) systems
        );
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              inputs.swift-overlay.overlays.default
            ];
          };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              pkgs.swift.bin."6.3.3" # Latest version
            ];
          };
        }
      );
    };
}
