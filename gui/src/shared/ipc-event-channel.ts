import { ICurrentAppVersionInfo } from '../main/index';
import { IWindowShapeParameters } from '../main/window-controller';
import { ILinuxSplitTunnelingApplication } from '../shared/application-types';
import {
  AccountToken,
  BridgeSettings,
  BridgeState,
  IAccountData,
  IAppVersionInfo,
  IDnsOptions,
  ILocation,
  IRelayList,
  ISettings,
  IWireguardPublicKey,
  KeygenEvent,
  RelaySettingsUpdate,
  TunnelState,
  VoucherResponse,
} from './daemon-rpc-types';
import { IGuiSettingsState } from './gui-settings-state';
import {
  createIpcMain,
  createIpcRenderer,
  invoke,
  invokeSync,
  notifyRenderer,
  send,
} from './ipc-helpers';

export interface IRelayListPair {
  relays: IRelayList;
  bridges: IRelayList;
}

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

// The different types of requests are:
// * send<ArgumentType>(), which is used for one-way communication from the renderer process to the
//    main process. The main channel will have a property named 'handle<PropertyName>' and the
//    renderer will have a property named the same as the one specified.
// * invoke<ArgumentType, ReturnType>(), which is used for two-way communication from the renderer
//    process to the main process. The naming is the same as `send<A>()`.
// * invokeSync<ArgumentType, ReturnType>(), same as `invoke<A, R>()` but synchronous.
// * notifyRenderer<ArgumentType>(), which is used for one-way communication from the main process
//    to the renderer process. The renderer ipc channel will have a property named
//    `listen<PropertyName>` and the main channel will have a property named `notify<PropertyName>`.
//
// Example:
// const ipc = {
//   groupOfCalls: {
//     first: send<boolean>(),
//     second: request<boolean, number>(),
//     third: requestSync<boolean, number>(),
//     fourth: notifyRenderer<boolean>(),
//   },
// };
//
// createIpcMain(ipc)
//   => {
//     groupOfCalls: {
//       handleFirst: (fn: (arg: boolean) => void) => void,
//       handleSecond: (fn: (arg: boolean) => Promise<number>) => void,
//       handleThird: (fn: (arg: boolean) => number) => void,
//       notifyFourth: (arg: boolean) => void,
//     },
//
// createIpcRenderer(ipc)
//   => {
//     groupOfCalls: {
//       first: (arg: boolean) => void,
//       second: (arg: boolean) => Promise<number>,
//       third: (arg: boolean) => number,
//       listenFourth: (fn: (arg: boolean) => void) => void,
//     },
//   }
const ipc = {
  state: {
    get: invokeSync<void, IAppStateSnapshot>(),
  },
  locale: {
    '': notifyRenderer<string>(),
  },
  windowShape: {
    '': notifyRenderer<IWindowShapeParameters>(),
  },
  windowFocus: {
    '': notifyRenderer<boolean>(),
  },
  daemonConnected: {
    '': notifyRenderer<void>(),
  },
  daemonDisconnected: {
    '': notifyRenderer<string | undefined>(),
  },
  location: {
    '': notifyRenderer<ILocation>(),
  },
  relays: {
    '': notifyRenderer<IRelayListPair>(),
  },
  currentVersion: {
    '': notifyRenderer<ICurrentAppVersionInfo>(),
  },
  upgradeVersion: {
    '': notifyRenderer<IAppVersionInfo>(),
  },
  app: {
    quit: send<void>(),
    openUrl: invoke<string, void>(),
    openPath: invoke<string, string>(),
    showOpenDialog: invoke<Electron.OpenDialogOptions, Electron.OpenDialogReturnValue>(),
  },
  tunnel: {
    '': notifyRenderer<TunnelState>(),
    connect: invoke<void, void>(),
    disconnect: invoke<void, void>(),
    reconnect: invoke<void, void>(),
  },
  settings: {
    '': notifyRenderer<ISettings>(),
    setAllowLan: invoke<boolean, void>(),
    setShowBetaReleases: invoke<boolean, void>(),
    setEnableIpv6: invoke<boolean, void>(),
    setBlockWhenDisconnected: invoke<boolean, void>(),
    setBridgeState: invoke<BridgeState, void>(),
    setOpenVpnMssfix: invoke<number | undefined, void>(),
    setWireguardMtu: invoke<number | undefined, void>(),
    updateRelaySettings: invoke<RelaySettingsUpdate, void>(),
    updateBridgeSettings: invoke<BridgeSettings, void>(),
    setDnsOptions: invoke<IDnsOptions, void>(),
  },
  guiSettings: {
    '': notifyRenderer<IGuiSettingsState>(),
    setEnableSystemNotifications: send<boolean>(),
    setAutoConnect: send<boolean>(),
    setStartMinimized: send<boolean>(),
    setMonochromaticIcon: send<boolean>(),
    setPreferredLocale: send<string>(),
    setUnpinnedWindow: send<boolean>(),
  },
  account: {
    '': notifyRenderer<IAccountData | undefined>(),
    create: invoke<void, string>(),
    login: invoke<AccountToken, void>(),
    logout: invoke<void, void>(),
    getWwwAuthToken: invoke<void, string>(),
    submitVoucher: invoke<string, VoucherResponse>(),
  },
  accountHistory: {
    '': notifyRenderer<AccountToken[]>(),
    removeItem: invoke<AccountToken, void>(),
  },
  autoStart: {
    '': notifyRenderer<boolean>(),
    set: invoke<boolean, void>(),
  },
  wireguardKeys: {
    publicKey: notifyRenderer<IWireguardPublicKey | undefined>(),
    keygenEvent: notifyRenderer<KeygenEvent>(),
    generateKey: invoke<void, KeygenEvent>(),
    verifyKey: invoke<void, boolean>(),
  },
  splitTunneling: {
    getApplications: invoke<void, ILinuxSplitTunnelingApplication[]>(),
    launchApplication: invoke<ILinuxSplitTunnelingApplication | string, void>(),
  },
  problemReport: {
    collectLogs: invoke<string[], string>(),
    sendReport: invoke<{ email: string; message: string; savedReport: string }, void>(),
  },
};

export const IpcMainEventChannel = createIpcMain(ipc);
export const IpcRendererEventChannel = createIpcRenderer(ipc);
