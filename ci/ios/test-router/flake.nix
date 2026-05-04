{
  description = "Config for our testing router";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs =
    { self, nixpkgs }:
    {
      nixosConfigurations.app-team-ios-lab = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          (import ./router-config.nix {
            hostname = "app-team-ios-tests";
            lanMac = "a0:ce:c8:ab:bd:2d";
            wanMac = "88:ae:dd:64:e1:55";
            lanIp = "192.168.105.1/24";
            wgIpv4 = "10.64.9.184/32";
            wgIpv6 = "fc00:bbbb:bbbb:bb01::a40:9b8/128";
          })
          ./app-team-ios-lab.nix
          {
            boot.loader.systemd-boot.enable = true;
            boot.loader.efi.canTouchEfiVariables = true;
            hardware = {
              cpu.intel.updateMicrocode = true;
              enableRedistributableFirmware = true;
            };
          }
        ];
      };

      nixosConfigurations.app-team-android-lab = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          (import ./router-config.nix {
            hostname = "app-team-android-tests";
            lanMac = "00:24:9b:8d:1e:f3";
            wanMac = "1c:69:7a:ad:73:a6";
            lanIp = "192.168.105.1/24";
            wgIpv4 = "10.64.17.146/32";
            wgIpv6 = "fc00:bbbb:bbbb:bb01::a40:1192/128";
          })
          ./app-team-android-lab.nix
          {
            boot.loader.systemd-boot.enable = true;
            boot.loader.efi.canTouchEfiVariables = true;
            hardware = {
              cpu.intel.updateMicrocode = true;
              enableRedistributableFirmware = true;
            };
          }
        ];
      };

      nixosConfigurations.app-team-ios-lab-iso = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          (import ./router-config.nix {
            hostname = "app-team-ios-tests";
            lanMac = "48:21:0b:36:bb:52";
            wanMac = "48:21:0b:36:43:a3";
            lanIp = "192.168.105.1/24";
            wgIpv4 = "10.64.9.184/32";
            wgIpv6 = "fc00:bbbb:bbbb:bb01::a40:9b8/128";
          })
          "${nixpkgs}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix"
          {
            isoImage.squashfsCompression = "lz4";
          }
        ];
      };

      packages.x86_64-linux.raas =
        with import nixpkgs { system = "x86_64-linux"; };
        pkgs.callPackage ./raas.nix { };
    };
}
