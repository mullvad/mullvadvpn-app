import { app, BrowserWindow } from 'electron';
import * as path from 'path';

import { getDefaultSettings } from '../../../src/main/default-settings';
import { changeIpcWebContents, IpcMainEventChannel } from '../../../src/main/ipc-event-channel';
import { loadTranslations } from '../../../src/main/load-translations';
import {
  DeviceState,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  IWireguardEndpointData,
} from '../../../src/shared/daemon-rpc-types';
import { messages, relayLocations } from '../../../src/shared/gettext';
import { IGuiSettingsState } from '../../../src/shared/gui-settings-state';
import { ITranslations, MacOsScrollbarVisibility } from '../../../src/shared/ipc-schema';
import { ICurrentAppVersionInfo } from '../../../src/shared/ipc-types';

const DEBUG = false;

class ApplicationMain {
  private guiSettings: IGuiSettingsState = {
    preferredLocale: 'en',
    autoConnect: false,
    enableSystemNotifications: true,
    monochromaticIcon: false,
    startMinimized: false,
    unpinnedWindow: process.platform !== 'win32' && process.platform !== 'darwin',
    browsedForSplitTunnelingApplications: [],
    changelogDisplayedForVersion: '',
  };

  private settings = getDefaultSettings();

  private translations: ITranslations = { locale: this.guiSettings.preferredLocale };

  private isConnectedToDaemon = true;

  private accountData: IAccountData = {
    expiry: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString(),
  };

  private deviceState: DeviceState = {
    type: 'logged in',
    accountAndDevice: {
      accountToken: '1234123412341234',
      device: {
        id: '1234',
        name: 'Testing Mole',
        ports: [],
        created: new Date(),
      },
    },
  };

  private currentVersion: ICurrentAppVersionInfo = {
    gui: '2000.1',
    daemon: '2000.1',
    isConsistent: true,
    isBeta: false,
  };
  private upgradeVersion: IAppVersionInfo = {
    supported: true,
    suggestedUpgrade: undefined,
  };

  private location: ILocation = {
    country: 'Sweden',
    city: 'Gothenburg',
    latitude: 58,
    longitude: 12,
    mullvadExitIp: false,
  };

  private relayList: IRelayList = {
    countries: [
      {
        name: 'Sweden',
        code: 'se',
        cities: [
          {
            name: 'Gothenburg',
            code: 'got',
            latitude: 58,
            longitude: 12,
            relays: [
              {
                hostname: 'se-got-wg-101',
                provider: 'mullvad',
                ipv4AddrIn: '127.0.0.1',
                includeInCountry: true,
                active: true,
                weight: 0,
                owned: true,
                endpointType: 'wireguard',
              },
            ],
          },
        ],
      },
    ],
  };

  private wireguardEndpointData: IWireguardEndpointData = {
    portRanges: [],
    udp2tcpPorts: [],
  };

  public constructor() {
    app.enableSandbox();
    app.on('ready', this.onReady);
  }

  private onReady = async () => {
    this.updateCurrentLocale('en');

    const window = new BrowserWindow({
      useContentSize: true,
      width: 320,
      height: 568,
      resizable: false,
      maximizable: false,
      fullscreenable: false,
      show: DEBUG,
      frame: true,
      webPreferences: {
        preload: path.join(__dirname, '../../../src/renderer/preloadBundle.js'),
        nodeIntegration: false,
        nodeIntegrationInWorker: false,
        nodeIntegrationInSubFrames: false,
        sandbox: true,
        contextIsolation: true,
        spellcheck: false,
        devTools: DEBUG,
      },
    });

    changeIpcWebContents(window.webContents);

    this.registerIpcListeners();

    const filePath = path.resolve(path.join(__dirname, '../../../src/renderer/index.html'));
    await window.loadFile(filePath);

    if (DEBUG) {
      window.webContents.openDevTools({ mode: 'detach' });
    }
  };

  private registerIpcListeners() {
    IpcMainEventChannel.state.handleGet(() => ({
      isConnected: this.isConnectedToDaemon,
      autoStart: false,
      accountData: this.accountData,
      accountHistory: undefined,
      tunnelState: { state: 'disconnected' },
      settings: this.settings,
      isPerformingPostUpgrade: false,
      deviceState: this.deviceState,
      relayListPair: {
        relays: this.relayList,
        bridges: this.relayList,
        wireguardEndpointData: this.wireguardEndpointData,
      },
      currentVersion: this.currentVersion,
      upgradeVersion: this.upgradeVersion,
      guiSettings: this.guiSettings,
      translations: this.translations,
      windowsSplitTunnelingApplications: [],
      macOsScrollbarVisibility: MacOsScrollbarVisibility.whenScrolling,
      changelog: [],
      forceShowChanges: false,
      navigationHistory: undefined,
      scrollPositions: {},
    }));

    IpcMainEventChannel.location.handleGet(() => Promise.resolve(this.location));

    IpcMainEventChannel.guiSettings.handleSetPreferredLocale((locale) => {
      this.updateCurrentLocale(locale);
      IpcMainEventChannel.guiSettings.notify?.(this.guiSettings);
      return Promise.resolve(this.translations);
    });
  }

  private updateCurrentLocale(locale: string) {
    this.guiSettings.preferredLocale = locale;

    const messagesTranslations = loadTranslations(this.guiSettings.preferredLocale, messages);
    const relayLocationsTranslations = loadTranslations(
      this.guiSettings.preferredLocale,
      relayLocations,
    );

    this.translations = {
      locale: this.guiSettings.preferredLocale,
      messages: messagesTranslations,
      relayLocations: relayLocationsTranslations,
    };
  }
}

new ApplicationMain();
