import { ipcMain, ipcRenderer, WebContents } from 'electron';
import * as uuid from 'uuid';

import { IGuiSettingsState } from './gui-settings-state';

import { IAppUpgradeInfo, ICurrentAppVersionInfo } from '../main/index';
import {
  AccountToken,
  IAccountData,
  ILocation,
  IRelayList,
  ISettings,
  RelaySettingsUpdate,
  TunnelStateTransition,
} from './daemon-rpc-types';

export interface IAppStateSnapshot {
  isConnected: boolean;
  autoStart: boolean;
  accountHistory: AccountToken[];
  tunnelState: TunnelStateTransition;
  settings: ISettings;
  location?: ILocation;
  relays: IRelayList;
  currentVersion: ICurrentAppVersionInfo;
  upgradeVersion: IAppUpgradeInfo;
  guiSettings: IGuiSettingsState;
}

interface ISender<T> {
  notify(webContents: WebContents, value: T): void;
}

interface ISenderVoid {
  notify(webContents: WebContents): void;
}

interface IReceiver<T> {
  listen(fn: (value: T) => void): void;
}

interface ITunnelMethods extends IReceiver<TunnelStateTransition> {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
}

interface ITunnelHandlers extends ISender<TunnelStateTransition> {
  handleConnect(fn: () => Promise<void>): void;
  handleDisconnect(fn: () => Promise<void>): void;
}

interface ISettingsMethods extends IReceiver<ISettings> {
  setAllowLan(allowLan: boolean): Promise<void>;
  setEnableIpv6(enableIpv6: boolean): Promise<void>;
  setBlockWhenDisconnected(block: boolean): Promise<void>;
  setOpenVpnMssfix(mssfix?: number): Promise<void>;
  updateRelaySettings(update: RelaySettingsUpdate): Promise<void>;
}

interface ISettingsHandlers extends ISender<ISettings> {
  handleAllowLan(fn: (allowLan: boolean) => Promise<void>): void;
  handleEnableIpv6(fn: (enableIpv6: boolean) => Promise<void>): void;
  handleBlockWhenDisconnected(fn: (block: boolean) => Promise<void>): void;
  handleOpenVpnMssfix(fn: (mssfix?: number) => Promise<void>): void;
  handleUpdateRelaySettings(fn: (update: RelaySettingsUpdate) => Promise<void>): void;
}

interface IGuiSettingsMethods extends IReceiver<IGuiSettingsState> {
  setAutoConnect(autoConnect: boolean): void;
  setStartMinimized(startMinimized: boolean): void;
  setMonochromaticIcon(monochromaticIcon: boolean): void;
}

interface IGuiSettingsHandlers extends ISender<IGuiSettingsState> {
  handleAutoConnect(fn: (autoConnect: boolean) => void): void;
  handleStartMinimized(fn: (startMinimized: boolean) => void): void;
  handleMonochromaticIcon(fn: (monochromaticIcon: boolean) => void): void;
}

interface IAccountHandlers {
  handleSet(fn: (token: AccountToken) => Promise<void>): void;
  handleUnset(fn: () => Promise<void>): void;
  handleGetData(fn: (token: AccountToken) => Promise<IAccountData>): void;
}

interface IAccountMethods {
  set(token: AccountToken): Promise<void>;
  unset(): Promise<void>;
  getData(token: AccountToken): Promise<IAccountData>;
}

interface IAccountHistoryHandlers extends ISender<AccountToken[]> {
  handleRemoveItem(fn: (token: AccountToken) => Promise<void>): void;
}

interface IAccountHistoryMethods extends IReceiver<AccountToken[]> {
  removeItem(token: AccountToken): Promise<void>;
}

interface IAutoStartMethods extends IReceiver<boolean> {
  set(autoStart: boolean): Promise<void>;
}

interface IAutoStartHandlers extends ISender<boolean> {
  handleSet(fn: (value: boolean) => Promise<void>): void;
}

/// Events names

const DAEMON_CONNECTED = 'daemon-connected';
const DAEMON_DISCONNECTED = 'daemon-disconnected';

const TUNNEL_STATE_CHANGED = 'tunnel-state-changed';
const CONNECT_TUNNEL = 'connect-tunnel';
const DISCONNECT_TUNNEL = 'disconnect-tunnel';

const SETTINGS_CHANGED = 'settings-changed';
const SET_ALLOW_LAN = 'set-allow-lan';
const SET_ENABLE_IPV6 = 'set-enable-ipv6';
const SET_BLOCK_WHEN_DISCONNECTED = 'set-block-when-disconnected';
const SET_OPENVPN_MSSFIX = 'set-openvpn-mssfix';
const UPDATE_RELAY_SETTINGS = 'update-relay-settings';

const LOCATION_CHANGED = 'location-changed';
const RELAYS_CHANGED = 'relays-changed';
const CURRENT_VERSION_CHANGED = 'current-version-changed';
const UPGRADE_VERSION_CHANGED = 'upgrade-version-changed';

const GUI_SETTINGS_CHANGED = 'gui-settings-changed';
const SET_AUTO_CONNECT = 'set-auto-connect';
const SET_MONOCHROMATIC_ICON = 'set-monochromatic-icon';
const SET_START_MINIMIZED = 'set-start-minimized';

const GET_APP_STATE = 'get-app-state';

const ACCOUNT_HISTORY_CHANGED = 'account-history-changed';
const REMOVE_ACCOUNT_HISTORY_ITEM = 'remove-account-history-item';

const SET_ACCOUNT = 'set-account';
const UNSET_ACCOUNT = 'unset-account';
const GET_ACCOUNT_DATA = 'get-account-data';

const AUTO_START_CHANGED = 'auto-start-changed';
const SET_AUTO_START = 'set-auto-start';

/// Typed IPC event channel
///
/// Static methods are meant to be provide the way to send the events from a renderer process, while
/// instance methods are meant to be used from a main process.
///
export class IpcRendererEventChannel {
  public static state = {
    /// Synchronously sends the IPC request and returns the app state snapshot
    get(): IAppStateSnapshot {
      return ipcRenderer.sendSync(GET_APP_STATE);
    },
  };

  public static daemonConnected: IReceiver<void> = {
    listen: listen(DAEMON_CONNECTED),
  };

  public static daemonDisconnected: IReceiver<string | undefined> = {
    listen: listen(DAEMON_DISCONNECTED),
  };

  public static tunnel: ITunnelMethods = {
    listen: listen(TUNNEL_STATE_CHANGED),
    connect: requestSender(CONNECT_TUNNEL),
    disconnect: requestSender(DISCONNECT_TUNNEL),
  };

  public static settings: ISettingsMethods = {
    listen: listen(SETTINGS_CHANGED),
    setAllowLan: requestSender(SET_ALLOW_LAN),
    setEnableIpv6: requestSender(SET_ENABLE_IPV6),
    setBlockWhenDisconnected: requestSender(SET_BLOCK_WHEN_DISCONNECTED),
    setOpenVpnMssfix: requestSender(SET_OPENVPN_MSSFIX),
    updateRelaySettings: requestSender(UPDATE_RELAY_SETTINGS),
  };

  public static location: IReceiver<ILocation> = {
    listen: listen(LOCATION_CHANGED),
  };

  public static relays: IReceiver<IRelayList> = {
    listen: listen(RELAYS_CHANGED),
  };

  public static currentVersion: IReceiver<ICurrentAppVersionInfo> = {
    listen: listen(CURRENT_VERSION_CHANGED),
  };

  public static upgradeVersion: IReceiver<IAppUpgradeInfo> = {
    listen: listen(UPGRADE_VERSION_CHANGED),
  };

  public static guiSettings: IGuiSettingsMethods = {
    listen: listen(GUI_SETTINGS_CHANGED),
    setAutoConnect: set(SET_AUTO_CONNECT),
    setMonochromaticIcon: set(SET_MONOCHROMATIC_ICON),
    setStartMinimized: set(SET_START_MINIMIZED),
  };

  public static autoStart: IAutoStartMethods = {
    listen: listen(AUTO_START_CHANGED),
    set: requestSender(SET_AUTO_START),
  };

  public static account: IAccountMethods = {
    set: requestSender(SET_ACCOUNT),
    unset: requestSender(UNSET_ACCOUNT),
    getData: requestSender(GET_ACCOUNT_DATA),
  };

  public static accountHistory: IAccountHistoryMethods = {
    listen: listen(ACCOUNT_HISTORY_CHANGED),
    removeItem: requestSender(REMOVE_ACCOUNT_HISTORY_ITEM),
  };
}

export class IpcMainEventChannel {
  public static state = {
    handleGet(fn: () => IAppStateSnapshot) {
      ipcMain.on(GET_APP_STATE, (event: Electron.Event) => {
        event.returnValue = fn();
      });
    },
  };

  public static daemonConnected: ISenderVoid = {
    notify: senderVoid(DAEMON_CONNECTED),
  };

  public static daemonDisconnected: ISender<string | undefined> = {
    notify: sender(DAEMON_DISCONNECTED),
  };

  public static tunnel: ITunnelHandlers = {
    notify: sender(TUNNEL_STATE_CHANGED),
    handleConnect: requestHandler(CONNECT_TUNNEL),
    handleDisconnect: requestHandler(DISCONNECT_TUNNEL),
  };

  public static location: ISender<ILocation> = {
    notify: sender(LOCATION_CHANGED),
  };

  public static settings: ISettingsHandlers = {
    notify: sender(SETTINGS_CHANGED),
    handleAllowLan: requestHandler(SET_ALLOW_LAN),
    handleEnableIpv6: requestHandler(SET_ENABLE_IPV6),
    handleBlockWhenDisconnected: requestHandler(SET_BLOCK_WHEN_DISCONNECTED),
    handleOpenVpnMssfix: requestHandler(SET_OPENVPN_MSSFIX),
    handleUpdateRelaySettings: requestHandler(UPDATE_RELAY_SETTINGS),
  };

  public static relays: ISender<IRelayList> = {
    notify: sender(RELAYS_CHANGED),
  };

  public static currentVersion: ISender<ICurrentAppVersionInfo> = {
    notify: sender(CURRENT_VERSION_CHANGED),
  };

  public static upgradeVersion: ISender<IAppUpgradeInfo> = {
    notify: sender(UPGRADE_VERSION_CHANGED),
  };

  public static guiSettings: IGuiSettingsHandlers = {
    notify: sender(GUI_SETTINGS_CHANGED),
    handleAutoConnect: handler(SET_AUTO_CONNECT),
    handleMonochromaticIcon: handler(SET_MONOCHROMATIC_ICON),
    handleStartMinimized: handler(SET_START_MINIMIZED),
  };

  public static autoStart: IAutoStartHandlers = {
    notify: sender<boolean>(AUTO_START_CHANGED),
    handleSet: requestHandler(SET_AUTO_START),
  };

  public static account: IAccountHandlers = {
    handleSet: requestHandler(SET_ACCOUNT),
    handleUnset: requestHandler(UNSET_ACCOUNT),
    handleGetData: requestHandler(GET_ACCOUNT_DATA),
  };

  public static accountHistory: IAccountHistoryHandlers = {
    notify: sender<AccountToken[]>(ACCOUNT_HISTORY_CHANGED),
    handleRemoveItem: requestHandler(REMOVE_ACCOUNT_HISTORY_ITEM),
  };
}

function listen<T>(event: string): (fn: (value: T) => void) => void {
  return (fn: (value: T) => void) => {
    ipcRenderer.on(event, (_event: Electron.Event, newState: T) => fn(newState));
  };
}

function set<T>(event: string): (value: T) => void {
  return (newValue: T) => {
    ipcRenderer.send(event, newValue);
  };
}

function sender<T>(event: string): (webContents: WebContents, value: T) => void {
  return (webContents: WebContents, value: T) => {
    webContents.send(event, value);
  };
}

function senderVoid(event: string): (webContents: WebContents) => void {
  return (webContents: WebContents) => {
    webContents.send(event);
  };
}

function handler<T>(event: string): (handlerFn: (value: T) => void) => void {
  return (handlerFn: (value: T) => void) => {
    ipcMain.on(event, (_event: Electron.Event, newValue: T) => {
      handlerFn(newValue);
    });
  };
}

type RequestResult<T> = { type: 'success'; value: T } | { type: 'error'; message: string };

function requestHandler<T>(event: string): (fn: (...args: any[]) => Promise<T>) => void {
  return (fn: (...args: any[]) => Promise<T>) => {
    ipcMain.on(event, async (ipcEvent: Electron.Event, requestId: string, ...args: any[]) => {
      const responseEvent = `${event}-${requestId}`;
      try {
        const result: RequestResult<T> = { type: 'success', value: await fn(...args) };

        ipcEvent.sender.send(responseEvent, result);
      } catch (error) {
        const result: RequestResult<T> = { type: 'error', message: error.message || '' };

        ipcEvent.sender.send(responseEvent, result);
      }
    });
  };
}

function requestSender<T>(event: string): (...args: any[]) => Promise<T> {
  return (...args: any[]): Promise<T> => {
    return new Promise((resolve: (result: T) => void, reject: (error: Error) => void) => {
      const requestId = uuid.v4();
      const responseEvent = `${event}-${requestId}`;

      ipcRenderer.once(responseEvent, (_ipcEvent: Electron.Event, result: RequestResult<T>) => {
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
