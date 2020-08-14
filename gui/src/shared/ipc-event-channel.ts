import { ipcMain, ipcRenderer, WebContents } from 'electron';
import log from 'electron-log';
import * as uuid from 'uuid';

import { IGuiSettingsState } from './gui-settings-state';

import { ICurrentAppVersionInfo } from '../main/index';
import { IWindowShapeParameters } from '../main/window-controller';
import ISplitTunnelingApplication from '../shared/linux-split-tunneling-application';
import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  IAccountData,
  IAppVersionInfo,
  ILocation,
  IRelayList,
  ISettings,
  IWireguardPublicKey,
  KeygenEvent,
  RelaySettingsUpdate,
  TunnelState,
  VoucherResponse,
} from './daemon-rpc-types';

export interface IAppStateSnapshot {
  locale: string;
  isConnected: boolean;
  autoStart: boolean;
  accountData?: IAccountData;
  accountHistory: AccountToken[];
  tunnelState: TunnelState;
  settings: ISettings;
  location?: ILocation;
  relayListPair: IRelayListPair;
  currentVersion: ICurrentAppVersionInfo;
  upgradeVersion: IAppVersionInfo;
  guiSettings: IGuiSettingsState;
  wireguardPublicKey?: IWireguardPublicKey;
}

export interface IRelayListPair {
  relays: IRelayList;
  bridges: IRelayList;
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

interface ITunnelMethods extends IReceiver<TunnelState> {
  connect(): Promise<void>;
  disconnect(): Promise<void>;
  reconnect(): Promise<void>;
}

interface ITunnelHandlers extends ISender<TunnelState> {
  handleConnect(fn: () => Promise<void>): void;
  handleDisconnect(fn: () => Promise<void>): void;
  handleReconnect(fn: () => Promise<void>): void;
}

interface ISettingsMethods extends IReceiver<ISettings> {
  setAllowLan(allowLan: boolean): Promise<void>;
  setShowBetaReleases(showBetaReleases: boolean): Promise<void>;
  setEnableIpv6(enableIpv6: boolean): Promise<void>;
  setBlockWhenDisconnected(block: boolean): Promise<void>;
  setBridgeState(state: BridgeState): Promise<void>;
  setOpenVpnMssfix(mssfix?: number): Promise<void>;
  setWireguardMtu(mtu?: number): Promise<void>;
  updateRelaySettings(update: RelaySettingsUpdate): Promise<void>;
  updateBridgeSettings(bridgeSettings: BridgeSettings): Promise<void>;
}

interface ISettingsHandlers extends ISender<ISettings> {
  handleAllowLan(fn: (allowLan: boolean) => Promise<void>): void;
  handleShowBetaReleases(fn: (showBetaReleases: boolean) => Promise<void>): void;
  handleEnableIpv6(fn: (enableIpv6: boolean) => Promise<void>): void;
  handleBlockWhenDisconnected(fn: (block: boolean) => Promise<void>): void;
  handleBridgeState(fn: (state: BridgeState) => Promise<void>): void;
  handleOpenVpnMssfix(fn: (mssfix?: number) => Promise<void>): void;
  handleWireguardMtu(fn: (mtu?: number) => Promise<void>): void;
  handleUpdateRelaySettings(fn: (update: RelaySettingsUpdate) => Promise<void>): void;
  handleUpdateBridgeSettings(fn: (bridgeSettings: BridgeSettings) => Promise<void>): void;
}

interface IGuiSettingsMethods extends IReceiver<IGuiSettingsState> {
  setEnableSystemNotifications(flag: boolean): void;
  setAutoConnect(autoConnect: boolean): void;
  setStartMinimized(startMinimized: boolean): void;
  setMonochromaticIcon(monochromaticIcon: boolean): void;
  setPreferredLocale(locale: string): void;
}

interface IGuiSettingsHandlers extends ISender<IGuiSettingsState> {
  handleEnableSystemNotifications(fn: (flag: boolean) => void): void;
  handleAutoConnect(fn: (autoConnect: boolean) => void): void;
  handleStartMinimized(fn: (startMinimized: boolean) => void): void;
  handleMonochromaticIcon(fn: (monochromaticIcon: boolean) => void): void;
  handleSetPreferredLocale(fn: (locale: string) => void): void;
}

interface IAccountHandlers extends ISender<IAccountData | undefined> {
  handleCreate(fn: () => Promise<string>): void;
  handleLogin(fn: (token: AccountToken) => Promise<void>): void;
  handleLogout(fn: () => Promise<void>): void;
  handleWwwAuthToken(fn: () => Promise<string>): void;
  handleSubmitVoucher(fn: (voucherCode: string) => Promise<VoucherResponse>): void;
}

interface IAccountMethods extends IReceiver<IAccountData | undefined> {
  create(): Promise<string>;
  login(token: AccountToken): Promise<void>;
  logout(): Promise<void>;
  getWwwAuthToken(): Promise<string>;
  submitVoucher(voucherCode: string): Promise<VoucherResponse>;
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

interface IWireguardKeyMethods extends IReceiver<IWireguardPublicKey | undefined> {
  listenKeygenEvents(fn: (event: KeygenEvent) => void): void;
  generateKey(): Promise<KeygenEvent>;
  verifyKey(): Promise<boolean>;
}

interface IWireguardKeyHandlers extends ISender<IWireguardPublicKey | undefined> {
  notifyKeygenEvent(webContents: WebContents, event: KeygenEvent): void;
  handleGenerateKey(fn: () => Promise<KeygenEvent>): void;
  handleVerifyKey(fn: () => Promise<boolean>): void;
}

interface ISplitTunnelingMethods {
  getApplications(): Promise<ISplitTunnelingApplication[]>;
  launchApplication(application: ISplitTunnelingApplication | string): Promise<void>;
}

interface ISplitTunnelingHandlers {
  handleGetApplications(fn: () => Promise<ISplitTunnelingApplication[]>): void;
  handleLaunchApplication(
    fn: (application: ISplitTunnelingApplication | string) => Promise<void>,
  ): void;
}

/// Events names

const LOCALE_CHANGED = 'locale-changed';
const WINDOW_SHAPE_CHANGED = 'window-shape-changed';

const DAEMON_CONNECTED = 'daemon-connected';
const DAEMON_DISCONNECTED = 'daemon-disconnected';

const TUNNEL_STATE_CHANGED = 'tunnel-state-changed';
const CONNECT_TUNNEL = 'connect-tunnel';
const DISCONNECT_TUNNEL = 'disconnect-tunnel';
const RECONNECT_TUNNEL = 'reconnect-tunnel';

const SETTINGS_CHANGED = 'settings-changed';
const SET_ALLOW_LAN = 'set-allow-lan';
const SET_SHOW_BETA_RELEASES = 'set-show-beta-releases';
const SET_ENABLE_IPV6 = 'set-enable-ipv6';
const SET_BLOCK_WHEN_DISCONNECTED = 'set-block-when-disconnected';
const SET_BRIDGE_STATE = 'set-bridge-state';
const SET_OPENVPN_MSSFIX = 'set-openvpn-mssfix';
const SET_WIREGUARD_MTU = 'set-wireguard-mtu';
const UPDATE_RELAY_SETTINGS = 'update-relay-settings';
const UPDATE_BRIDGE_SETTINGS = 'update-bridge-location';

const LOCATION_CHANGED = 'location-changed';
const RELAYS_CHANGED = 'relays-changed';
const CURRENT_VERSION_CHANGED = 'current-version-changed';
const UPGRADE_VERSION_CHANGED = 'upgrade-version-changed';

const GUI_SETTINGS_CHANGED = 'gui-settings-changed';
const SET_ENABLE_SYSTEM_NOTIFICATIONS = 'set-enable-system-notifications';
const SET_AUTO_CONNECT = 'set-auto-connect';
const SET_MONOCHROMATIC_ICON = 'set-monochromatic-icon';
const SET_START_MINIMIZED = 'set-start-minimized';
const SET_PREFERRED_LOCALE = 'set-preferred-locale';

const GET_APP_STATE = 'get-app-state';

const ACCOUNT_HISTORY_CHANGED = 'account-history-changed';
const REMOVE_ACCOUNT_HISTORY_ITEM = 'remove-account-history-item';

const CREATE_NEW_ACCOUNT = 'create-new-account';
const DO_LOGIN = 'do-login';
const DO_LOGOUT = 'do-logout';
const DO_GET_WWW_AUTH_TOKEN = 'do-get-www-auth-token';
const ACCOUNT_DATA_CHANGED = 'account-data-changed';
const REDEEM_VOUCHER = 'redeem-voucher';

const AUTO_START_CHANGED = 'auto-start-changed';
const SET_AUTO_START = 'set-auto-start';

const WIREGUARD_KEY_SET = 'wireguard-key-change-event';
const WIREGUARD_KEYGEN_EVENT = 'wireguard-keygen-event';
const GENERATE_WIREGUARD_KEY = 'generate-wireguard-key';
const VERIFY_WIREGUARD_KEY = 'verify-wireguard-key';

const SPLIT_TUNNELING_GET_APPLICATIONS = 'split-tunneling-get-applications';
const SPLIT_TUNNELING_LAUNCH_APPLICATION = 'split-tunneling-launch-application';

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

  public static locale: IReceiver<string> = {
    listen: listen(LOCALE_CHANGED),
  };

  public static windowShape: IReceiver<IWindowShapeParameters> = {
    listen: listen(WINDOW_SHAPE_CHANGED),
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
    reconnect: requestSender(RECONNECT_TUNNEL),
  };

  public static settings: ISettingsMethods = {
    listen: listen(SETTINGS_CHANGED),
    setAllowLan: requestSender(SET_ALLOW_LAN),
    setShowBetaReleases: requestSender(SET_SHOW_BETA_RELEASES),
    setEnableIpv6: requestSender(SET_ENABLE_IPV6),
    setBlockWhenDisconnected: requestSender(SET_BLOCK_WHEN_DISCONNECTED),
    setBridgeState: requestSender(SET_BRIDGE_STATE),
    setOpenVpnMssfix: requestSender(SET_OPENVPN_MSSFIX),
    setWireguardMtu: requestSender(SET_WIREGUARD_MTU),
    updateRelaySettings: requestSender(UPDATE_RELAY_SETTINGS),
    updateBridgeSettings: requestSender(UPDATE_BRIDGE_SETTINGS),
  };

  public static location: IReceiver<ILocation> = {
    listen: listen(LOCATION_CHANGED),
  };

  public static relays: IReceiver<IRelayListPair> = {
    listen: listen(RELAYS_CHANGED),
  };

  public static currentVersion: IReceiver<ICurrentAppVersionInfo> = {
    listen: listen(CURRENT_VERSION_CHANGED),
  };

  public static upgradeVersion: IReceiver<IAppVersionInfo> = {
    listen: listen(UPGRADE_VERSION_CHANGED),
  };

  public static guiSettings: IGuiSettingsMethods = {
    listen: listen(GUI_SETTINGS_CHANGED),
    setEnableSystemNotifications: set(SET_ENABLE_SYSTEM_NOTIFICATIONS),
    setAutoConnect: set(SET_AUTO_CONNECT),
    setMonochromaticIcon: set(SET_MONOCHROMATIC_ICON),
    setStartMinimized: set(SET_START_MINIMIZED),
    setPreferredLocale: set(SET_PREFERRED_LOCALE),
  };

  public static autoStart: IAutoStartMethods = {
    listen: listen(AUTO_START_CHANGED),
    set: requestSender(SET_AUTO_START),
  };

  public static account: IAccountMethods = {
    listen: listen(ACCOUNT_DATA_CHANGED),
    create: requestSender(CREATE_NEW_ACCOUNT),
    login: requestSender(DO_LOGIN),
    logout: requestSender(DO_LOGOUT),
    getWwwAuthToken: requestSender(DO_GET_WWW_AUTH_TOKEN),
    submitVoucher: requestSender(REDEEM_VOUCHER),
  };

  public static accountHistory: IAccountHistoryMethods = {
    listen: listen(ACCOUNT_HISTORY_CHANGED),
    removeItem: requestSender(REMOVE_ACCOUNT_HISTORY_ITEM),
  };

  public static wireguardKeys: IWireguardKeyMethods = {
    listen: listen(WIREGUARD_KEY_SET),
    listenKeygenEvents: listen(WIREGUARD_KEYGEN_EVENT),
    generateKey: requestSender(GENERATE_WIREGUARD_KEY),
    verifyKey: requestSender(VERIFY_WIREGUARD_KEY),
  };

  public static splitTunneling: ISplitTunnelingMethods = {
    getApplications: requestSender(SPLIT_TUNNELING_GET_APPLICATIONS),
    launchApplication: requestSender(SPLIT_TUNNELING_LAUNCH_APPLICATION),
  };
}

export class IpcMainEventChannel {
  public static state = {
    handleGet(fn: () => IAppStateSnapshot) {
      ipcMain.on(GET_APP_STATE, (event: Electron.IpcMainEvent) => {
        event.returnValue = fn();
      });
    },
  };

  public static locale: ISender<string> = {
    notify: sender(LOCALE_CHANGED),
  };

  public static windowShape: ISender<IWindowShapeParameters> = {
    notify: sender(WINDOW_SHAPE_CHANGED),
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
    handleReconnect: requestHandler(RECONNECT_TUNNEL),
  };

  public static location: ISender<ILocation> = {
    notify: sender(LOCATION_CHANGED),
  };

  public static settings: ISettingsHandlers = {
    notify: sender(SETTINGS_CHANGED),
    handleAllowLan: requestHandler(SET_ALLOW_LAN),
    handleShowBetaReleases: requestHandler(SET_SHOW_BETA_RELEASES),
    handleEnableIpv6: requestHandler(SET_ENABLE_IPV6),
    handleBlockWhenDisconnected: requestHandler(SET_BLOCK_WHEN_DISCONNECTED),
    handleBridgeState: requestHandler(SET_BRIDGE_STATE),
    handleOpenVpnMssfix: requestHandler(SET_OPENVPN_MSSFIX),
    handleWireguardMtu: requestHandler(SET_WIREGUARD_MTU),
    handleUpdateRelaySettings: requestHandler(UPDATE_RELAY_SETTINGS),
    handleUpdateBridgeSettings: requestHandler(UPDATE_BRIDGE_SETTINGS),
  };

  public static relays: ISender<IRelayListPair> = {
    notify: sender(RELAYS_CHANGED),
  };

  public static currentVersion: ISender<ICurrentAppVersionInfo> = {
    notify: sender(CURRENT_VERSION_CHANGED),
  };

  public static upgradeVersion: ISender<IAppVersionInfo> = {
    notify: sender(UPGRADE_VERSION_CHANGED),
  };

  public static guiSettings: IGuiSettingsHandlers = {
    notify: sender(GUI_SETTINGS_CHANGED),
    handleEnableSystemNotifications: handler(SET_ENABLE_SYSTEM_NOTIFICATIONS),
    handleAutoConnect: handler(SET_AUTO_CONNECT),
    handleMonochromaticIcon: handler(SET_MONOCHROMATIC_ICON),
    handleStartMinimized: handler(SET_START_MINIMIZED),
    handleSetPreferredLocale: handler(SET_PREFERRED_LOCALE),
  };

  public static autoStart: IAutoStartHandlers = {
    notify: sender<boolean>(AUTO_START_CHANGED),
    handleSet: requestHandler(SET_AUTO_START),
  };

  public static account: IAccountHandlers = {
    notify: sender<IAccountData | undefined>(ACCOUNT_DATA_CHANGED),
    handleCreate: requestHandler(CREATE_NEW_ACCOUNT),
    handleLogin: requestHandler(DO_LOGIN),
    handleLogout: requestHandler(DO_LOGOUT),
    handleWwwAuthToken: requestHandler(DO_GET_WWW_AUTH_TOKEN),
    handleSubmitVoucher: requestHandler<VoucherResponse>(REDEEM_VOUCHER),
  };

  public static accountHistory: IAccountHistoryHandlers = {
    notify: sender<AccountToken[]>(ACCOUNT_HISTORY_CHANGED),
    handleRemoveItem: requestHandler(REMOVE_ACCOUNT_HISTORY_ITEM),
  };

  public static wireguardKeys: IWireguardKeyHandlers = {
    notify: sender<IWireguardPublicKey | undefined>(WIREGUARD_KEY_SET),
    notifyKeygenEvent: sender<KeygenEvent>(WIREGUARD_KEYGEN_EVENT),
    handleGenerateKey: requestHandler(GENERATE_WIREGUARD_KEY),
    handleVerifyKey: requestHandler(VERIFY_WIREGUARD_KEY),
  };

  public static splitTunneling: ISplitTunnelingHandlers = {
    handleGetApplications: requestHandler(SPLIT_TUNNELING_GET_APPLICATIONS),
    handleLaunchApplication: requestHandler(SPLIT_TUNNELING_LAUNCH_APPLICATION),
  };
}

function listen<T>(event: string): (fn: (value: T) => void) => void {
  return (fn: (value: T) => void) => {
    ipcRenderer.on(event, (_event: Electron.IpcRendererEvent, newState: T) => fn(newState));
  };
}

function set<T>(event: string): (value: T) => void {
  return (newValue: T) => {
    ipcRenderer.send(event, newValue);
  };
}

function sender<T>(event: string): (webContents: WebContents, value: T) => void {
  return (webContents: WebContents, value: T) => {
    if (webContents.isDestroyed()) {
      log.error(`sender(${event}): webContents is already destroyed!`);
    } else {
      webContents.send(event, value);
    }
  };
}

function senderVoid(event: string): (webContents: WebContents) => void {
  return (webContents: WebContents) => {
    if (webContents.isDestroyed()) {
      log.error(`senderVoid(${event}): webContents is already destroyed!`);
    } else {
      webContents.send(event);
    }
  };
}

function handler<T>(event: string): (handlerFn: (value: T) => void) => void {
  return (handlerFn: (value: T) => void) => {
    ipcMain.on(event, (_event: Electron.IpcMainEvent, newValue: T) => {
      handlerFn(newValue);
    });
  };
}

type RequestResult<T> = { type: 'success'; value: T } | { type: 'error'; message: string };

// The Electron API uses the `any` type.
/* eslint-disable @typescript-eslint/no-explicit-any */
function requestHandler<T>(event: string): (fn: (...args: any[]) => Promise<T>) => void {
  return (fn: (...args: any[]) => Promise<T>) => {
    ipcMain.on(
      event,
      async (ipcEvent: Electron.IpcMainEvent, requestId: string, ...args: any[]) => {
        const responseEvent = `${event}-${requestId}`;
        try {
          const result: RequestResult<T> = { type: 'success', value: await fn(...args) };

          if (ipcEvent.sender.isDestroyed()) {
            log.debug(`Cannot send the reply for ${responseEvent} since the sender was destroyed.`);
          } else {
            ipcEvent.sender.send(responseEvent, result);
          }
        } catch (error) {
          const result: RequestResult<T> = { type: 'error', message: error.message || '' };

          if (ipcEvent.sender.isDestroyed()) {
            log.debug(`Cannot send the reply for ${responseEvent} since the sender was destroyed.`);
          } else {
            ipcEvent.sender.send(responseEvent, result);
          }
        }
      },
    );
  };
}
/* eslint-enable @typescript-eslint/no-explicit-any */

// The Electron API uses the `any` type.
/* eslint-disable @typescript-eslint/no-explicit-any */
function requestSender<T>(event: string): (...args: any[]) => Promise<T> {
  return (...args: any[]): Promise<T> => {
    return new Promise((resolve: (result: T) => void, reject: (error: Error) => void) => {
      const requestId = uuid.v4();
      const responseEvent = `${event}-${requestId}`;

      ipcRenderer.once(
        responseEvent,
        (_ipcEvent: Electron.IpcRendererEvent, result: RequestResult<T>) => {
          switch (result.type) {
            case 'error':
              reject(new Error(result.message));
              break;

            case 'success':
              resolve(result.value);
              break;
          }
        },
      );

      ipcRenderer.send(event, requestId, ...args);
    });
  };
}
/* eslint-enable @typescript-eslint/no-explicit-any */
