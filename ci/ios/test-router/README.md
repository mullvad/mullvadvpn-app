# Router setup
## How to use `RAAS` on a developer machine

> [!CAUTION]
> For safety, the binary must be built as a regular user, then moved to folder on your path and
> lastly changed to be owned by `root`.
> Executing `cargo` in a root context is **not** considered safe.

### Installing
> [!TIP]
> If you have `nix` installed, executing `nix profile install .#raas` should be sufficient.

You'll be executing the following steps:

1. Build raas

    - `cd raas && cargo build`

2. Install it into `.local/bin` and `chown`/`chmod` to the root user

    - You can substitute `.local/bin` with another folder that is on your `PATH`
    - `sudo install -o root -g wheel -m 755 -t ~/.local/bin target/debug/raas`

3. Mark the binary as immutable by the user

    - `sudo chflags uchg "$INSTALL_PATH/raas"`
    - `ucgh` means: "User immutable" as in the file cannot be changed, renamed, or deleted, even by the owner.
      The flag can be removed by the owner.

The entire sequence put together looks similar to this:

```sh
export INSTALL_PATH="$HOME/.local/bin"
cd raas
cargo build
sudo install -o root -g wheel -m 755 target/debug/raas "$INSTALL_PATH"
sudo chflags uchg "$INSTALL_PATH/raas"
sudo -k
```


> [!IMPORTANT]
> If you later want to update `raas`, use `sudo chflags nouchg "$INSTALL_PATH"` prior
> to the `install` step to unmark it as such.

### Executing

#### MacOS
Both the machine running `raas` and the device under test are expected to be on the same network.
On MacOS, `raas` takes 1 parameter: the address on which to listen. This includes IP and port, meaning something like:

```sh
# Use the IP reachable by your mobile device
sudo raas <your_interface_ipv4_address>:80
```

On the device under test, use the IP you used in the above command as the device's gateway (also called `router` on iOS).
After that, you are ready to execute the end-to-end tests locally 🎉.

#### On iPhone:
- Go to `Settings > Wi-Fi`
- Tap the network information icon

  - Observe your current configuration

- Navigate to `Configure IP` and choose `Manual`

  1. Enter the previously observed IP and subnet for your network
  2. Enter the IP on which the phone can reach `raas` into the phone's `router` field
  3. Tap `Save`

- Navigate to the `Configure DNS` and choose `Manual`

  1. Tap `Add Server` and enter `1.1.1.1`
  2. Tap `Save`

- Open  `Safari` and go to https://mullvad.net to confirm connectivity

🎉 You can now execute the end-to-end tests on your phone 🎉

### Troubleshooting

1. The phone is not able to connect to the internet

    1. Enable `Airplane mode` and enable Wi-Fi.
    2. Check whether the `Wi-Fi` icon is shown, and internet to be available

        - If not, something is amiss with the phone's configuration
          Confirm the `DNS` configuration is set to `Manual` and a DNS server was added

    3. Check whether the computer can ping the phone

        - If not, check whether you are connected to the same Wi-Fi network

    4. Execute `sudo tcpdump -nn -i <network_interface> host <phone_ip>` to check whether traffic is reaching the computer.
       `en0` is usually a MacBook's Wi-Fi interface. An example: `sudo tcpdump -nn -i en0 host 192.168.101.253`

        - If not, ask for help :')

## How-to to configure `RAAS` on a new router/computer
- Obtain an x86 computer with 2 ethernet interfaces.
- Install NixOS on the hardware following the [NixOS installation guide]
- Copy the generated `/etc/nixos/hardware-config.nix` file to the flake repo, add it to git.
- Create a new Stagemole account, and add plentry of time to it
  * Go to stagemole.eu, log in with the account number and generate a wireguard configuration with a wg
  private key and wg interface IP addresses.
- Add a new _nixosConfiguration_ entry in `flake.nix`, following `app-team-ios-lab` as an example, making sure to import
    the hardware config.
    * Be sure to include the `hardware-config.nix` file as it contains the mount config for the partitions.
      * Set the appropriate args for the `./router-config.nix` import, as to not clash with existing SSIDs.
      Also set the `wgIpv4` and `wgIpv6` args to the IP addresses from the wireguard config.

- Apply the new configuration either via SSH or by copying the flake over to the nix machine
  * `nixos-reubild switch .#$newMachine --target-host root@$newMachine-ip` if one can SSH into the machine
  * `nixos-reubild switch .$pathToFlake#$newMachine` if flake is copied to nix machine, with `$pathToFlake` being the
      path to this flake directory.
- Copy the wireguard private key from the generated config to the file `/staging-wg-private-key`

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
```

And build it like so:
`nix build .#nixosConfigurations.app-team-ios-lab-iso.config.system.build.isoImage`


## Quirks & features
- Since Apple doesn't allow access to LAN without the user accepting a privacy
  dialog, TCP connections to `8.8.8.8:80` are NAT'ed to the gateway address.


[NixOS installation guide]: https://nixos.org/manual/nixos/stable/#sec-installation-graphical
