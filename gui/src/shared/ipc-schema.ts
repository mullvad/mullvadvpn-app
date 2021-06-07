import { GetTextTranslations } from 'gettext-parser';
import { IApplication, ILinuxSplitTunnelingApplication } from './application-types';
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
import { LogLevel } from './logging-types';

interface ILogEntry {
  level: LogLevel;
  message: string;
}
import { invoke, invokeSync, notifyRenderer, send } from './ipc-helpers';
import { ICurrentAppVersionInfo, IWindowShapeParameters } from './ipc-types';

export interface ITranslations {
  locale: string;
  messages?: GetTextTranslations;
  relayLocations?: GetTextTranslations;
}

export interface IRelayListPair {
  relays: IRelayList;
  bridges: IRelayList;
}

export type LaunchApplicationResult = { success: true } | { error: string };

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
  translations: ITranslations;
  platform: NodeJS.Platform;
  runningInDevelopment: boolean;
  windowsSplitTunnelingApplications?: IApplication[];
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
export const ipcSchema = {
  state: {
    get: invokeSync<void, IAppStateSnapshot>(),
  },
  windowShape: {
    '': notifyRenderer<IWindowShapeParameters>(),
  },
  windowFocus: {
    '': notifyRenderer<boolean>(),
  },
  daemon: {
    connected: notifyRenderer<void>(),
    disconnected: notifyRenderer<void>(),
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
    setPreferredLocale: invoke<string, ITranslations>(),
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
  problemReport: {
    collectLogs: invoke<string[], string>(),
    sendReport: invoke<{ email: string; message: string; savedReportId: string }, void>(),
    viewLog: invoke<string, string>(),
  },
  logging: {
    log: send<ILogEntry>(),
  },
  linuxSplitTunneling: {
    getApplications: invoke<void, ILinuxSplitTunnelingApplication[]>(),
    launchApplication: invoke<ILinuxSplitTunnelingApplication | string, LaunchApplicationResult>(),
  },
  windowsSplitTunneling: {
    '': notifyRenderer<IApplication[]>(),
    setState: invoke<boolean, void>(),
    getApplications: invoke<boolean, { fromCache: boolean; applications: IApplication[] }>(),
    addApplication: invoke<IApplication | string, void>(),
    removeApplication: invoke<IApplication | string, void>(),
  },
};
