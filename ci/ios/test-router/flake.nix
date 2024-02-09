{
  description = "Config for our testing router";

  inputs = { nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11"; };

  outputs = flake@{ self, nixpkgs }: {
    nixosConfigurations.app-team-ios-lab = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        (import ./router-config.nix {
          ssid = "app-team-ios-tests";
          lanMac = "48:21:0b:36:bb:52";
          wanMac = "48:21:0b:36:43:a3";
          wifiMac = "bc:6e:e2:a8:38:51";
          lanIp = "192.168.105.1/24";
        })
        ./app-team-ios-lab.nix
      ];
    };

    nixosConfigurations.app-team-ios-lab-iso = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        (import ./router-config.nix {
          ssid = "app-team-ios-tests";
          lanMac = "48:21:0b:36:bb:52";
          wanMac = "48:21:0b:36:43:a3";
          wifiMac = "bc:6e:e2:a8:38:51";
          lanIp = "192.168.105.1/24";
        })
        "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix"
        {
          isoImage.squashfsCompression = "lz4";
        }
      ];
    };

    packages.x86_64-linux.raas =
      with import nixpkgs { system = "x86_64-linux"; };
      pkgs.callPackage ./raas.nix {};

  };
}
