import BridgeSettingsBuilder from '../shared/bridge-settings-builder';
import { ISettings, ObfuscationType, Ownership } from '../shared/daemon-rpc-types';
import log from '../shared/logging';
import { setOpenAtLogin } from './autostart';
import { DaemonRpc } from './daemon-rpc';
import { IpcMainEventChannel } from './ipc-event-channel';

export interface SettingsDelegate {
  updateDaemonsAutoConnect(): void;
}

export default class Settings implements Readonly<ISettings> {
  private settingsValue: ISettings = {
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
      },
      dns: {
        state: 'default',
        defaultOptions: {
          blockAds: false,
          blockTrackers: false,
          blockMalware: false,
          blockAdultContent: false,
          blockGambling: false,
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
  };

  public constructor(private delegate: SettingsDelegate, private daemonRpc: DaemonRpc) {}

  public registerIpcListeners() {
    IpcMainEventChannel.settings.handleSetAllowLan((allowLan) =>
      this.daemonRpc.setAllowLan(allowLan),
    );
    IpcMainEventChannel.settings.handleSetShowBetaReleases((showBetaReleases) =>
      this.daemonRpc.setShowBetaReleases(showBetaReleases),
    );
    IpcMainEventChannel.settings.handleSetEnableIpv6((enableIpv6) =>
      this.daemonRpc.setEnableIpv6(enableIpv6),
    );
    IpcMainEventChannel.settings.handleSetBlockWhenDisconnected((blockWhenDisconnected) =>
      this.daemonRpc.setBlockWhenDisconnected(blockWhenDisconnected),
    );
    IpcMainEventChannel.settings.handleSetBridgeState(async (bridgeState) => {
      await this.daemonRpc.setBridgeState(bridgeState);

      // Reset bridge constraints to `any` when the state is set to auto or off
      if (bridgeState === 'auto' || bridgeState === 'off') {
        await this.daemonRpc.setBridgeSettings(new BridgeSettingsBuilder().location.any().build());
      }
    });
    IpcMainEventChannel.settings.handleSetOpenVpnMssfix((mssfix?: number) =>
      this.daemonRpc.setOpenVpnMssfix(mssfix),
    );
    IpcMainEventChannel.settings.handleSetWireguardMtu((mtu?: number) =>
      this.daemonRpc.setWireguardMtu(mtu),
    );
    IpcMainEventChannel.settings.handleUpdateRelaySettings((update) =>
      this.daemonRpc.updateRelaySettings(update),
    );
    IpcMainEventChannel.settings.handleUpdateBridgeSettings((bridgeSettings) => {
      return this.daemonRpc.setBridgeSettings(bridgeSettings);
    });
    IpcMainEventChannel.settings.handleSetDnsOptions((dns) => {
      return this.daemonRpc.setDnsOptions(dns);
    });
    IpcMainEventChannel.autoStart.handleSet((autoStart: boolean) => {
      return this.setAutoStart(autoStart);
    });
    IpcMainEventChannel.settings.handleSetObfuscationSettings((obfuscationSettings) => {
      return this.daemonRpc.setObfuscationSettings(obfuscationSettings);
    });
  }

  public get all() {
    return this.settingsValue;
  }

  public get allowLan() {
    return this.settingsValue.allowLan;
  }
  public get autoConnect() {
    return this.settingsValue.autoConnect;
  }
  public get blockWhenDisconnected() {
    return this.settingsValue.blockWhenDisconnected;
  }
  public get showBetaReleases() {
    return this.settingsValue.showBetaReleases;
  }
  public get relaySettings() {
    return this.settingsValue.relaySettings;
  }
  public get tunnelOptions() {
    return this.settingsValue.tunnelOptions;
  }
  public get bridgeSettings() {
    return this.settingsValue.bridgeSettings;
  }
  public get bridgeState() {
    return this.settingsValue.bridgeState;
  }
  public get splitTunnel() {
    return this.settingsValue.splitTunnel;
  }
  public get obfuscationSettings() {
    return this.settingsValue.obfuscationSettings;
  }

  public handleNewSettings(newSettings: ISettings) {
    this.settingsValue = newSettings;
  }

  private async setAutoStart(autoStart: boolean): Promise<void> {
    try {
      await setOpenAtLogin(autoStart);

      IpcMainEventChannel.autoStart.notify?.(autoStart);

      this.delegate.updateDaemonsAutoConnect();
    } catch (e) {
      const error = e as Error;
      log.error(
        `Failed to update the autostart to ${autoStart.toString()}. ${error.message.toString()}`,
      );
    }
    return Promise.resolve();
  }
}
