args@{ ssid, # SSID of the WiFi network, also the hostname
lanMac, # MAC address of the local area network interface
wanMac, # MAC address of the upstream interface
wifiMac, # MAC address of the WiFi interface
lanIp, # IP adderss/subnet
}:
{ config, pkgs, lib, ... }:

let
  raas = pkgs.callPackage ./raas.nix {};

  gatewayIpGroup = builtins.match "([0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+)/[0-9]+" args.lanIp;
  gatewayAddress = builtins.elemAt gatewayIpGroup 0;
in
{
  imports = [
    ./nftables.nix
  ];

  services.nftables.internetHostOverride = gatewayAddress;

  environment.systemPackages = with pkgs; [ htop vim curl dig tcpdump cargo ];

  boot.extraModulePackages = with config.boot.kernelPackages; [ rtl8812au ];
  boot.loader.systemd-boot.enable = true;
  boot.loader.systemd-boot.memtest86.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking.hostName = args.ssid;
  networking.useDHCP = true;

  system.stateVersion = "23.11";

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";

  systemd.network.netdevs."1-lanBridge" = {
    netdevConfig = {
      Kind = "bridge";
      Name = "lan";
    };
  };

  systemd.network.links."1-lanIface" = {
    matchConfig.PermanentMACAddress = args.lanMac;
    linkConfig.Name = "lanEth";
  };

  systemd.network.links."1-wanIface" = {
    matchConfig.PermanentMACAddress = args.wanMac;
    linkConfig.Name = "wan";
  };

  systemd.network.links."1-wifiIface" = {
    matchConfig.PermanentMACAddress = args.wifiMac;
    linkConfig.Name = "wifi";
  };
  networking = { firewall.enable = false; };
  hardware.bluetooth.enable = false;

  boot.kernel.sysctl = {
    # if you use ipv4, this is all you need
    "net.ipv4.conf.all.forwarding" = true;

    # If you want to use it for ipv6
    "net.ipv6.conf.all.forwarding" = true;

    # source: https://github.com/mdlayher/homelab/blob/master/nixos/routnerr-2/configuration.nix#L52
    # By default, not automatically configure any IPv6 addresses.
    "net.ipv6.conf.default.accept_ra" = 0;
    "net.ipv6.conf.default.autoconf" = 0;
  };

  # when the above sysctl script is executed, wan is not renamed yet
  systemd.services.sysctl_fixup_after_boot = {
    enable = true;
    bindsTo = [ "sys-subsystem-net-devices-wan.device" ];
    before = [ "systemd-networkd.service" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig.ExecStart = ''
      ${pkgs.sysctl}/bin/sysctl net.ipv6.conf.wan.accept_ra=2 net.ipv6.conf.wan.autoconf=1 net.ipv6.conf.all.use_tempaddr=1
    '';
  };

  networking.useNetworkd = true;
  systemd.network.enable = true;

  systemd.network.networks.wan = {
    name = "wan";
    DHCP = "yes";

    networkConfig = {
      IPv6AcceptRA = true;
      DHCP = "yes";
    };

    ipv6AcceptRAConfig = {
      DHCPv6Client = "always";
      UseDNS = true;
    };

    dhcpV4Config = {
      Hostname = args.ssid;
      UseDNS = true;
    };

    dhcpV6Config = { UseDNS = true; };
  };

  # obtain all leases
  # if=wifi; \
  # link_id="$(ip --oneline link show dev "$if" | cut -f 1 -d:)"; \
  # busctl -j get-property org.freedesktop.network1 \
  #  "/org/freedesktop/network1/link/${link_id}" \
  #  org.freedesktop.network1.DHCPServer \
  #  Leases

  systemd.network.networks."lanEth" = {
    matchConfig.Name = "lanEth";
    networkConfig.Bridge = "lan";
    linkConfig.RequiredForOnline = "enslaved";
  };
  systemd.network.networks."wifi" = {
    matchConfig.Name = "wifi";
    networkConfig.Bridge = "lan";
    linkConfig.RequiredForOnline = "enslaved";
  };

  systemd.network.networks.lan = {
    name = "lan";
    address = [ "192.168.105.1/24" ];

    networkConfig = {
      DHCPServer = true;
      IPv6AcceptRA = false;
      IPv6SendRA = true;
      DHCPPrefixDelegation = true;
      ConfigureWithoutCarrier = true;
    };

    dhcpServerConfig = {
      ServerAddress = "192.168.105.1/24";
      DNS = [ "1.1.1.1" "1.0.0.1" ];
      PoolOffset = 128;
      EmitDNS = true;
      EmitNTP = true;
      UplinkInterface = "wan";
    };

    dhcpServerStaticLeases = [
      # {
      # If we ever want a specific MAC to receive a static IP, add them here :)
      # dhcpServerStaticLeaseConfig = {
      #   Address = "192.168.105.2";
      #   MACAddress = "78:45:58:C3:75:5E";
      # };
      # }
    ];

    ipv6SendRAConfig = {
      UplinkInterface = [ "wan" ];
      EmitDNS = true;
    };

    dhcpPrefixDelegationConfig = {
      UplinkInterface = "wan";
      Announce = true;
      Assign = true;
    };
  };

  services.resolved.enable = true;

  hardware = {
    cpu.intel.updateMicrocode = true;
    enableRedistributableFirmware = true;
  };

  services.hostapd.enable = true;
  systemd.services.hostapd = {
    bindsTo = [ "sys-subsystem-net-devices-wifi.device" ];
  };
  services.hostapd.radios.wifi = {
    wifi5.enable = false;
    wifi4.capabilities = [ "HT40+" "HT40-" "HT20" "SHORT-GI-20" "SHORT-GI-40" ];

    countryCode = "SE";
    band = "2g";
    networks.wifi = {
      # the regular NixOS config is too strict w.r.t. to old WPA standards, so for increased compatibility we should use this.
      settings = {
        "channel" = lib.mkForce "7";
        "driver" = lib.mkForce "nl80211";
        "ht_capab" =
          lib.mkForce "[HT40+][HT40-][HT20][SHORT-GI-20][SHORT-GI-40]";
        "hw_mode" = lib.mkForce "g";
        "ieee80211w" = lib.mkForce "1";
        "ieee80211d" = lib.mkForce "1";
        "ieee80211h" = lib.mkForce "1";
        "ieee80211n" = lib.mkForce "1";
        "noscan" = lib.mkForce "0";
        "require_ht" = lib.mkForce "0";
        "wpa_key_mgmt" = lib.mkForce "WPA-PSK WPA-PSK-SHA256 SAE";
        "group_mgmt_cipher" = lib.mkForce "AES-128-CMAC";
      };
      ssid = args.ssid;
      authentication = {
        mode = "wpa2-sha256";
        # ¡¡¡ CREATE THESE FILES WITH THE NECESSARY PASSWORD !!!
        wpaPasswordFile = "/wifi-password";
        saePasswordsFile = "/wifi-sae-passwords";
      };
    };
  };

  # disable logging forever
  services.journald.extraConfig = ''
    Storage=Volatile;
    SystemMaxUse=50M
    RuntimeMaxUse=50M
  '';

  services.openssh = {
    enable = true;
    ports = [ 22 2021 ];
    settings.PermitRootLogin = "yes";
  };

  services.avahi = {
    enable = true;
    reflector = true;
    allowInterfaces = [ "lan" ];
  };

  systemd.services.raas =
  let
    listenIpGroup = builtins.match "([0-9]+\\.[0-9]+\\.[0-9]+\\.[0-9]+)/[0-9]+" args.lanIp;
    listenAddress = builtins.elemAt listenIpGroup 0;
  in {
    enable = true;
    description = "Web service to apply blocking firewall rules";
    bindsTo = [ "sys-subsystem-net-devices-lan.device" ];
    after = [ "systemd-networkd.service" "network-online.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig.ExecStart = ''
      ${raas}/bin/raas ${listenAddress}:80
    '';
  };
}
