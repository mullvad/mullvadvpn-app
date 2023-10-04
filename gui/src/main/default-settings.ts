import { ISettings, ObfuscationType, Ownership } from '../shared/daemon-rpc-types';

export function getDefaultSettings(): ISettings {
  return {
    allowLan: false,
    autoConnect: false,
    blockWhenDisconnected: false,
    showBetaReleases: false,
    splitTunnel: {
      enableExclusions: false,
      appsList: [],
    },
    relaySettings: {
      normal: {
        location: 'any',
        tunnelProtocol: 'any',
        providers: [],
        ownership: Ownership.any,
        openvpnConstraints: {
          port: 'any',
          protocol: 'any',
        },
        wireguardConstraints: {
          port: 'any',
          ipVersion: 'any',
          useMultihop: false,
          entryLocation: 'any',
        },
      },
    },
    bridgeSettings: {
      normal: {
        location: 'any',
        providers: [],
        ownership: Ownership.any,
      },
    },
    bridgeState: 'auto',
    tunnelOptions: {
      generic: {
        enableIpv6: false,
      },
      openvpn: {
        mssfix: undefined,
      },
      wireguard: {
        mtu: undefined,
        quantumResistant: undefined,
      },
      dns: {
        state: 'default',
        defaultOptions: {
          blockAds: false,
          blockTrackers: false,
          blockMalware: false,
          blockAdultContent: false,
          blockGambling: false,
          blockSocialMedia: false,
        },
        customOptions: {
          addresses: [],
        },
      },
    },
    obfuscationSettings: {
      selectedObfuscation: ObfuscationType.auto,
      udp2tcpSettings: {
        port: 'any',
      },
    },
    customLists: [],
  };
}
