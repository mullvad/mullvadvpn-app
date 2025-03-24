{ lib, config, ... }:
with lib; let
  cfg = config.services.nftables;
in
{
  options.services.nftables.internetHostOverride = mkOption {
    type = types.str;
    default = false;
    description = ''
      Gateway address to which traffic to 8.8.8.8:80 will be forwarded to.
    '';
  };

  options.services.nftables.lanInterfaces = mkOption {
    type = types.str;
    default = false;
    description = ''
      A string representing the interfaces on the LAN side of the network.
    '';
  };

  config.systemd.services.nftables = {
    before = lib.mkForce [ ];
    bindsTo = [
      "sys-subsystem-net-devices-wan.device"
      "sys-subsystem-net-devices-lan.device"
    ];
    after = [
      "systemd-networkd.service"
      "sys-subsystem-net-devices-wan.device"
      "sys-subsystem-net-devices-lan.device"
    ];
  };

  config.networking.nftables = {
    enable = true;
    preCheckRuleset = ''
      sed 's/lan/lo/g' -i ruleset.conf
      sed 's/wan/lo/g' -i ruleset.conf
      sed 's/wifi/lo/g' -i ruleset.conf
    '';
    ruleset = ''
      table inet filter {
        chain output {
          type filter hook output priority 100; policy accept;
        }


        chain input {
           type filter hook input priority filter; policy drop;

           # allow reaching systemd-resolve
           ip saddr 127.0.0.1 ip daddr 127.0.0.53 accept
           iifname lo accept;
           oifname lo accept;
           # Allow trusted networks to access the router
           iifname { lan } counter accept;
           # Allow WiFi clients reach the following:
           # - DNS
           # - DHCP
           # - DHCPv6
           iifname { wan, ${cfg.lanInterfaces} } udp dport 53 counter accept
           iifname { wan, ${cfg.lanInterfaces} } tcp dport 53 counter accept
           iifname { wifi } udp sport 68 udp dport 67 counter accept
           iifname { wifi } ip6 saddr fe80::/10 udp sport 546 ip6 daddr fe80::/10 udp dport 547 accept

           iifname wan meta nfproto ipv6 accept


           # allow SSH from WAN
           iifname "wan" tcp dport 2021 counter accept
           # allow WG from WAN
           iifname "wan" udp dport 6070 counter accept


           # allow random traffic for testing purposes
           iifname "wan" udp dport {9090, 9091} counter accept
           iifname "wan" tcp dport {9090, 9091} counter accept

           iifname { "wan", "staging" } ct state vmap { established : accept, related : accept, invalid : drop }
           iifname "wan" udp sport 67 udp dport 68 counter accept;
           iifname "wan" ip6 saddr fe80::/10 udp sport 547 ip6 daddr fe80::/10 udp dport 546 counter accept

           icmpv6 code no-route counter accept
           iifname "wan" icmpv6 mtu > 0 counter accept comment "Allow ALL ICMP from wan"
           icmpv6 type { nd-neighbor-solicit, nd-router-advert, nd-neighbor-advert } counter accept

        }

        chain forward {
          type filter hook forward priority filter; policy drop;

          ip daddr 192.168.0.0/23 drop

          # offload established HTTP connections
          # ip protocol { tcp, udp } ct state established flow offload @internetNat counter

          # Allow traffic from established and related packets, drop invalid
          ct state vmap { established : accept, related : accept, invalid : drop }

          # Allow trusted network WAN access
          iifname {
                  lo, ${cfg.lanInterfaces}
          } oifname {
                  "wan", "staging"
          } counter accept comment "Allow trusted LAN to WAN and staging interface"

          iifname "lan" oifname "wifi" counter accept comment "Allow LAN to IoS WiFi"

          # Allow established WAN to return
          iifname { "wan", "wifi" } oifname { ${cfg.lanInterfaces} } ct state established,related counter accept comment "Allow established back to LANs"
          iifname {"wan", "staging" } oifname { "lan", "wifi"} ct mark 1919 accept comment "Allow DNAtted traffic"
        }

        chain srcnat {
          type nat hook postrouting priority srcnat; policy accept;
          iifname { ${cfg.lanInterfaces} }  masquerade comment "Masquerade all traffic"
        }

        chain dstnat {
          type nat hook prerouting priority dstnat; policy accept;
          ip daddr 8.8.8.8 tcp dport 80 dnat to ${cfg.internetHostOverride};
          # host of the bridge IP address
          ip daddr 85.203.53.200 tcp dport 443 dnat to ${cfg.internetHostOverride};
        }
      }
    '';
  };
}
