// @flow

import { ipcMain, ipcRenderer } from 'electron';
import type { WebContents } from 'electron';

import type { GuiSettingsState } from './gui-settings-state';

import type { AppUpgradeInfo, CurrentAppVersionInfo } from '../main/index';
import type { Location, RelayList, Settings, TunnelStateTransition } from '../main/daemon-rpc';

export type AppStateSnapshot = {
  isConnected: boolean,
  tunnelState: TunnelStateTransition,
  settings: Settings,
  location: ?Location,
  relays: RelayList,
  currentVersion: CurrentAppVersionInfo,
  upgradeVersion: AppUpgradeInfo,
  guiSettings: GuiSettingsState,
};

interface Sender<T> {
  notify(newState: T): void;
}

interface Receiver<T> {
  listen<T>(fn: (T) => void): void;
}

interface GuiSettingsMethods {
  setStartMinimized: (boolean) => void;
  setUncoupledFromTunnel: (boolean) => void;
}

interface GuiSettingsHandlers {
  handleStartMinimized: ((boolean) => void) => void;
  handleUncoupledFromTunnel: ((boolean) => void) => void;
}

/// Events names

const DAEMON_CONNECTED = 'daemon-connected';
const DAEMON_DISCONNECTED = 'daemon-disconnected';
const TUNNEL_STATE_CHANGED = 'tunnel-state-changed';
const SETTINGS_CHANGED = 'settings-changed';
const LOCATION_CHANGED = 'location-changed';
const RELAYS_CHANGED = 'relays-changed';
const CURRENT_VERSION_CHANGED = 'current-version-changed';
const UPGRADE_VERSION_CHANGED = 'upgrade-version-changed';
const GUI_SETTINGS_CHANGED = 'gui-settings-changed';

const SET_START_MINIMIZED = 'set-start-minimized';
const SET_UNCOUPLED_FROM_TUNNEL = 'set-uncoupled-from-tunnel';

/// Typed IPC event channel
///
/// Static methods are meant to be provide the way to send the events from a renderer process, while
/// instance methods are meant to be used from a main process.
///
export default class IpcEventChannel {
  _webContents: WebContents;

  constructor(webContents: WebContents) {
    this._webContents = webContents;
  }

  static state = {
    /// Should be used from the main process to process state snapshot requests
    serve(fn: () => AppStateSnapshot) {
      ipcMain.on('get-state', (event) => {
        event.returnValue = fn();
      });
    },

    /// Synchronously sends the IPC request and returns the app state snapshot
    get(): AppStateSnapshot {
      return ipcRenderer.sendSync('get-state');
    },
  };

  static daemonConnected: Receiver<void> = {
    listen: listen(DAEMON_CONNECTED),
  };

  get daemonConnected(): Sender<void> {
    return {
      notify: sender(this._webContents, DAEMON_CONNECTED),
    };
  }

  static daemonDisconnected: Receiver<?string> = {
    listen: listen(DAEMON_DISCONNECTED),
  };

  get daemonDisconnected(): Sender<?string> {
    return {
      notify: sender(this._webContents, DAEMON_DISCONNECTED),
    };
  }

  static tunnelState: Receiver<TunnelStateTransition> = {
    listen: listen(TUNNEL_STATE_CHANGED),
  };

  get tunnelState(): Sender<TunnelStateTransition> {
    return {
      notify: sender(this._webContents, TUNNEL_STATE_CHANGED),
    };
  }

  static settings: Receiver<Settings> = {
    listen: listen(SETTINGS_CHANGED),
  };

  get settings(): Sender<Settings> {
    return {
      notify: sender(this._webContents, SETTINGS_CHANGED),
    };
  }

  static location: Receiver<Location> = {
    listen: listen(LOCATION_CHANGED),
  };

  get location(): Sender<Location> {
    return {
      notify: sender(this._webContents, LOCATION_CHANGED),
    };
  }

  static relays: Receiver<RelayList> = {
    listen: listen(RELAYS_CHANGED),
  };

  get relays(): Sender<RelayList> {
    return {
      notify: sender(this._webContents, RELAYS_CHANGED),
    };
  }

  static currentVersion: Receiver<CurrentAppVersionInfo> = {
    listen: listen(CURRENT_VERSION_CHANGED),
  };

  get currentVersion(): Sender<CurrentAppVersionInfo> {
    return {
      notify: sender(this._webContents, CURRENT_VERSION_CHANGED),
    };
  }

  static upgradeVersion: Receiver<AppUpgradeInfo> = {
    listen: listen(UPGRADE_VERSION_CHANGED),
  };

  get upgradeVersion(): Sender<AppUpgradeInfo> {
    return {
      notify: sender(this._webContents, UPGRADE_VERSION_CHANGED),
    };
  }

  static guiSettings: Receiver<GuiSettingsState> & GuiSettingsMethods = {
    listen: listen(GUI_SETTINGS_CHANGED),
    setStartMinimized: set(SET_START_MINIMIZED),
    setUncoupledFromTunnel: set(SET_UNCOUPLED_FROM_TUNNEL),
  };

  get guiSettings(): Sender<GuiSettingsState> & GuiSettingsHandlers {
    return {
      notify: sender(this._webContents, GUI_SETTINGS_CHANGED),
      handleStartMinimized: handler(SET_START_MINIMIZED),
      handleUncoupledFromTunnel: handler(SET_UNCOUPLED_FROM_TUNNEL),
    };
  }
}

function listen<T>(event: string): ((T) => void) => void {
  return function(fn: (T) => void) {
    ipcRenderer.on(event, (_, newState: T) => fn(newState));
  };
}

function set<T>(event: string): (T) => void {
  return function(newValue: T) {
    ipcRenderer.send(event, newValue);
  };
}

function sender<T>(webContents: WebContents, event: string): (T) => void {
  return function(newState: T) {
    webContents.send(event, newState);
  };
}

function handler<T>(event: string): ((T) => void) => void {
  return function(handlerFn: (T) => void) {
    ipcMain.on(event, (_, newValue: T) => {
      handlerFn(newValue);
    });
  };
}
