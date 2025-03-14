# Router setup
## Installing on a new router/computer
- Obtain an x86 computer with 2 ethernet and 1 WiFi interface.
- Install NixOS on the hardware following the [NixOS installation guide]
- Copy the generated `/etc/nixos/hardware-config.nix` file to the flake repo, add it to git.
- Add a new _nixosConfiguration_ entry in `flake.nix`, following `app-team-ios-lab` as an example, making sure to import
    the hardware config.
    * Be sure to include the `hardware-config.nix` file as it contains the mount config for the partitions.
    * Set the appropriate args for the `./router-config.nix` import, as to not clash with existing SSIDs.

- Create `/wifi-password` and `/wifi-sae-passwords` files on the router, they should contain an ASCII password of 8-63
    characters in length.
- Apply the new configuration either via SSH or by copying the flake over to the nix machine
  * `nixos-reubild switch .#$newMachine --target-host root@$newMachine-ip` if one can SSH into the machine
  * `nixos-reubild switch .$pathToFlake#$newMachine` if flake is copied to nix machine, with `$pathToFlake` being the
      path to this flake directory.

## Livebooting
One can create an ISO to live-boot a router needing to permanently install this config. There are two drawbacks:
* Still need to know the MAC addresses of the interfaces upfront.
* Any updates to the running system will not persist.

To do this, add a `nixosConfiguration` with an extra import of the installer ISO profile like so:
```nix
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
```

And build it like so:
`nix build .#nixosConfigurations.app-team-ios-lab-iso.config.system.build.isoImage`


## Quirks & features
- Since Apple doesn't allow access to LAN without the user accepting a privacy
  dialog, TCP connections to `8.8.8.8:80` are NAT'ed to the gateway address.




[NixOS installation guide]: https://nixos.org/manual/nixos/stable/#sec-installation-graphical
[`wifi-password`]: https://search.nixos.org/options?channel=23.11&show=services.hostapd.radios.%3Cname%3E.networks.%3Cname%3E.authentication.wpaPasswordFile&from=0&size=50&sort=relevance&type=packages&query=hostapd+password
[`wifi-sae-password`]: https://search.nixos.org/options?channel=23.11&show=services.hostapd.radios.%3Cname%3E.networks.%3Cname%3E.authentication.saePasswordsFile&from=0&size=50&sort=relevance&type=packages&query=hostapd+password

