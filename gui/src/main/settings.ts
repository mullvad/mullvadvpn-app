import BridgeSettingsBuilder from '../shared/bridge-settings-builder';
import { ISettings } from '../shared/daemon-rpc-types';
import { ICurrentAppVersionInfo } from '../shared/ipc-types';
import log from '../shared/logging';
import { getOpenAtLogin, setOpenAtLogin } from './autostart';
import { DaemonRpc } from './daemon-rpc';
import { getDefaultSettings } from './default-settings';
import GuiSettings from './gui-settings';
import { IpcMainEventChannel } from './ipc-event-channel';

export interface SettingsDelegate {
  handleMonochromaticIconChange(value: boolean): void;
  handleUnpinnedWindowChange(): void;
}

export default class Settings implements Readonly<ISettings> {
  private guiSettings = new GuiSettings();

  private settingsValue = getDefaultSettings();

  public constructor(
    private delegate: SettingsDelegate,
    private daemonRpc: DaemonRpc,
    private currentVersion: ICurrentAppVersionInfo,
  ) {}

  public registerIpcListeners() {
    this.registerGuiSettingsListener();

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
    IpcMainEventChannel.settings.handleSetWireguardQuantumResistant((quantumResistant?: boolean) =>
      this.daemonRpc.setWireguardQuantumResistant(quantumResistant),
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

    IpcMainEventChannel.guiSettings.handleSetEnableSystemNotifications((flag: boolean) => {
      this.guiSettings.enableSystemNotifications = flag;
    });

    IpcMainEventChannel.guiSettings.handleSetAutoConnect((autoConnect: boolean) => {
      this.guiSettings.autoConnect = autoConnect;
    });

    IpcMainEventChannel.guiSettings.handleSetStartMinimized((startMinimized: boolean) => {
      this.guiSettings.startMinimized = startMinimized;
    });

    IpcMainEventChannel.guiSettings.handleSetMonochromaticIcon((monochromaticIcon: boolean) => {
      this.guiSettings.monochromaticIcon = monochromaticIcon;
    });

    IpcMainEventChannel.guiSettings.handleSetUnpinnedWindow((unpinnedWindow: boolean) => {
      this.guiSettings.unpinnedWindow = unpinnedWindow;
      this.delegate.handleUnpinnedWindowChange();
    });

    IpcMainEventChannel.currentVersion.handleDisplayedChangelog(() => {
      this.guiSettings.changelogDisplayedForVersion = this.currentVersion.gui;
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
  public get customLists() {
    return this.settingsValue.customLists;
  }

  public get gui() {
    return this.guiSettings;
  }

  public handleNewSettings(newSettings: ISettings) {
    this.settingsValue = newSettings;
  }

  private registerGuiSettingsListener() {
    this.guiSettings.onChange = (newState, oldState) => {
      if (oldState.monochromaticIcon !== newState.monochromaticIcon) {
        this.delegate.handleMonochromaticIconChange(newState.monochromaticIcon);
      }

      if (newState.autoConnect !== oldState.autoConnect) {
        this.updateDaemonsAutoConnect();
      }

      IpcMainEventChannel.guiSettings.notify?.(newState);
    };
  }

  private async setAutoStart(autoStart: boolean): Promise<void> {
    try {
      await setOpenAtLogin(autoStart);

      IpcMainEventChannel.autoStart.notify?.(autoStart);

      this.updateDaemonsAutoConnect();
    } catch (e) {
      const error = e as Error;
      log.error(
        `Failed to update the autostart to ${autoStart.toString()}. ${error.message.toString()}`,
      );
    }
    return Promise.resolve();
  }

  private updateDaemonsAutoConnect() {
    const daemonAutoConnect = this.guiSettings.autoConnect && getOpenAtLogin();
    if (daemonAutoConnect !== this.settingsValue.autoConnect) {
      void this.daemonRpc.setAutoConnect(daemonAutoConnect);
    }
  }
}
