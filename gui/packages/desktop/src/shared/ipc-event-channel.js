// @flow

import { ipcMain, ipcRenderer } from 'electron';
import type { WebContents, IpcMainEvent, IpcRendererEvent } from 'electron';
import uuid from 'uuid';

import type { GuiSettingsState } from './gui-settings-state';

import type { AppUpgradeInfo, CurrentAppVersionInfo } from '../main/index';
import type {
  AccountToken,
  Location,
  RelayList,
  Settings,
  TunnelStateTransition,
} from '../main/daemon-rpc';

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
  notify(webContents: WebContents, newState: T): void;
}

interface Receiver<T> {
  listen<T>(fn: (T) => void): void;
}

interface GuiSettingsMethods {
  setStartMinimized: (boolean) => void;
  setMonochromaticIcon: (boolean) => void;
}

interface GuiSettingsHandlers {
  handleStartMinimized: ((boolean) => void) => void;
  handleMonochromaticIcon: ((boolean) => void) => void;
}

interface AccountHistoryHandlers {
  handleGet(fn: () => Promise<Array<AccountToken>>): void;
  handleRemoveItem(fn: (token: AccountToken) => Promise<void>): void;
}

interface AccountHistoryMethods {
  get(): Promise<Array<AccountToken>>;
  removeItem(token: AccountToken): Promise<void>;
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

const SET_MONOCHROMATIC_ICON = 'set-monochromatic-icon';
const SET_START_MINIMIZED = 'set-start-minimized';
const GET_APP_STATE = 'get-app-state';
const GET_ACCOUNT_HISTORY = 'get-account-history';
const REMOVE_ACCOUNT_HISTORY_ITEM = 'remove-account-history-item';

/// Typed IPC event channel
///
/// Static methods are meant to be provide the way to send the events from a renderer process, while
/// instance methods are meant to be used from a main process.
///
export class IpcRendererEventChannel {
  static state = {
    /// Synchronously sends the IPC request and returns the app state snapshot
    get(): AppStateSnapshot {
      return ipcRenderer.sendSync(GET_APP_STATE);
    },
  };

  static daemonConnected: Receiver<void> = {
    listen: listen(DAEMON_CONNECTED),
  };

  static daemonDisconnected: Receiver<?string> = {
    listen: listen(DAEMON_DISCONNECTED),
  };

  static tunnelState: Receiver<TunnelStateTransition> = {
    listen: listen(TUNNEL_STATE_CHANGED),
  };

  static settings: Receiver<Settings> = {
    listen: listen(SETTINGS_CHANGED),
  };

  static location: Receiver<Location> = {
    listen: listen(LOCATION_CHANGED),
  };

  static relays: Receiver<RelayList> = {
    listen: listen(RELAYS_CHANGED),
  };

  static currentVersion: Receiver<CurrentAppVersionInfo> = {
    listen: listen(CURRENT_VERSION_CHANGED),
  };

  static upgradeVersion: Receiver<AppUpgradeInfo> = {
    listen: listen(UPGRADE_VERSION_CHANGED),
  };

  static guiSettings: Receiver<GuiSettingsState> & GuiSettingsMethods = {
    listen: listen(GUI_SETTINGS_CHANGED),
    setMonochromaticIcon: set(SET_MONOCHROMATIC_ICON),
    setStartMinimized: set(SET_START_MINIMIZED),
  };

  static accountHistory: AccountHistoryMethods = {
    get: requestSender(GET_ACCOUNT_HISTORY),
    removeItem: requestSender(REMOVE_ACCOUNT_HISTORY_ITEM),
  };
}

export class IpcMainEventChannel {
  static state = {
    handleGet(fn: () => AppStateSnapshot) {
      ipcMain.on(GET_APP_STATE, (event) => {
        event.returnValue = fn();
      });
    },
  };

  static daemonConnected: Sender<void> = {
    notify: sender(DAEMON_CONNECTED),
  };

  static daemonDisconnected: Sender<?string> = {
    notify: sender(DAEMON_DISCONNECTED),
  };

  static tunnelState: Sender<TunnelStateTransition> = {
    notify: sender(TUNNEL_STATE_CHANGED),
  };

  static location: Sender<Location> = {
    notify: sender(LOCATION_CHANGED),
  };

  static settings: Sender<Settings> = {
    notify: sender(SETTINGS_CHANGED),
  };

  static relays: Sender<RelayList> = {
    notify: sender(RELAYS_CHANGED),
  };

  static currentVersion: Sender<CurrentAppVersionInfo> = {
    notify: sender(CURRENT_VERSION_CHANGED),
  };

  static upgradeVersion: Sender<AppUpgradeInfo> = {
    notify: sender(UPGRADE_VERSION_CHANGED),
  };

  static guiSettings: Sender<GuiSettingsState> & GuiSettingsHandlers = {
    notify: sender(GUI_SETTINGS_CHANGED),
    handleMonochromaticIcon: handler(SET_MONOCHROMATIC_ICON),
    handleStartMinimized: handler(SET_START_MINIMIZED),
  };

  static accountHistory: AccountHistoryHandlers = {
    handleGet: requestHandler(GET_ACCOUNT_HISTORY),
    handleRemoveItem: requestHandler(REMOVE_ACCOUNT_HISTORY_ITEM),
  };
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

function sender<T>(event: string): (WebContents, T) => void {
  return function(webContents: WebContents, newState: T) {
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

type RequestResult<T> = { type: 'success', value: T } | { type: 'error', message: string };

function requestHandler<T>(event: string): (fn: (...args: Array<any>) => Promise<T>) => void {
  return function(fn: (...args: Array<any>) => Promise<T>) {
    ipcMain.on(event, async (ipcEvent: IpcMainEvent, requestId: string, ...args: Array<any>) => {
      const sender = ipcEvent.sender;
      const responseEvent = `${event}-${requestId}`;
      try {
        const result: RequestResult<T> = { type: 'success', value: await fn(...args) };

        sender.send(responseEvent, result);
      } catch (error) {
        const result: RequestResult<T> = { type: 'error', message: error.message || '' };

        sender.send(responseEvent, result);
      }
    });
  };
}

function requestSender<T>(event: string): (...args: Array<any>) => Promise<T> {
  return function(...args: Array<any>): Promise<T> {
    return new Promise((resolve: (result: T) => void, reject: (error: Error) => void) => {
      const requestId = uuid.v4();
      const responseEvent = `${event}-${requestId}`;

      ipcRenderer.once(responseEvent, (_ipcEvent: IpcRendererEvent, result: RequestResult<T>) => {
        switch (result.type) {
          case 'error':
            reject(new Error(result.message));
            break;

          case 'success':
            resolve(result.value);
            break;
        }
      });

      ipcRenderer.send(event, requestId, ...args);
    });
  };
}
