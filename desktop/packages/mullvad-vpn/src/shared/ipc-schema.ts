import { GetTextTranslations } from 'gettext-parser';

import { ILinuxSplitTunnelingApplication, ISplitTunnelingApplication } from './application-types';
import {
  AccessMethodSetting,
  AccountDataError,
  AccountNumber,
  BridgeSettings,
  BridgeState,
  CustomListError,
  CustomProxy,
  DeviceEvent,
  DeviceState,
  IAccountData,
  IAppVersionInfo,
  ICustomList,
  IDevice,
  IDeviceRemoval,
  IDnsOptions,
  IRelayListWithEndpointData,
  ISettings,
  NewAccessMethodSetting,
  ObfuscationSettings,
  RelaySettings,
  TunnelState,
  VoucherResponse,
} from './daemon-rpc-types';
import { IGuiSettingsState } from './gui-settings-state';
import { LogLevel } from './logging-types';

interface ILogEntry {
  level: LogLevel;
  message: string;
}
import { MapData } from '../renderer/lib/3dmap';
import { AppUpgradeEvent } from './app-upgrade';
import { AppUpgradeError } from './constants';
import { invoke, invokeSync, notifyRenderer, send } from './ipc-helpers';
import {
  IChangelog,
  ICurrentAppVersionInfo,
  IHistoryObject,
  IWindowShapeParameters,
} from './ipc-types';

export interface ITranslations {
  locale: string;
  messages?: GetTextTranslations;
  relayLocations?: GetTextTranslations;
}

export type LaunchApplicationResult = { success: true } | { error: string };

export enum MacOsScrollbarVisibility {
  always,
  whenScrolling,
  automatic,
}

export interface IAppStateSnapshot {
  isConnected: boolean;
  autoStart: boolean;
  accountData?: IAccountData;
  accountHistory?: AccountNumber;
  tunnelState: TunnelState;
  settings: ISettings;
  isPerformingPostUpgrade: boolean;
  daemonAllowed?: boolean;
  deviceState?: DeviceState;
  relayList?: IRelayListWithEndpointData;
  currentVersion: ICurrentAppVersionInfo;
  upgradeVersion: IAppVersionInfo;
  guiSettings: IGuiSettingsState;
  translations: ITranslations;
  splitTunnelingApplications?: ISplitTunnelingApplication[];
  macOsScrollbarVisibility?: MacOsScrollbarVisibility;
  changelog: IChangelog;
  navigationHistory?: IHistoryObject;
  currentApiAccessMethod?: AccessMethodSetting;
  isMacOs13OrNewer: boolean;
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
  map: {
    getData: invoke<void, MapData>(),
  },
  window: {
    shape: notifyRenderer<IWindowShapeParameters>(),
    focus: notifyRenderer<boolean>(),
    macOsScrollbarVisibility: notifyRenderer<MacOsScrollbarVisibility>(),
    scaleFactorChange: notifyRenderer<void>(),
  },
  navigation: {
    reset: notifyRenderer<void>(),
    setHistory: send<IHistoryObject>(),
  },
  daemon: {
    isPerformingPostUpgrade: notifyRenderer<boolean>(),
    daemonAllowed: notifyRenderer<boolean>(),
    connected: notifyRenderer<void>(),
    disconnected: notifyRenderer<void>(),
    prepareRestart: send<boolean>(),
  },
  relays: {
    '': notifyRenderer<IRelayListWithEndpointData>(),
  },
  customLists: {
    createCustomList: invoke<string, void | CustomListError>(),
    deleteCustomList: invoke<string, void>(),
    updateCustomList: invoke<ICustomList, void | CustomListError>(),
  },
  currentVersion: {
    '': notifyRenderer<ICurrentAppVersionInfo>(),
    displayedChangelog: send<void>(),
  },
  upgradeVersion: {
    '': notifyRenderer<IAppVersionInfo>(),
    dismissedUpgrade: send<string>(),
  },
  app: {
    quit: send<void>(),
    openUrl: invoke<string, void>(),
    showOpenDialog: invoke<Electron.OpenDialogOptions, Electron.OpenDialogReturnValue>(),
    showLaunchDaemonSettings: invoke<void, void>(),
    showFullDiskAccessSettings: invoke<void, void>(),
    getPathBaseName: invoke<string, string>(),
    upgrade: send<void>(),
    upgradeAbort: send<void>(),
    upgradeEvent: notifyRenderer<AppUpgradeEvent>(),
    upgradeError: notifyRenderer<AppUpgradeError>(),
  },
  tunnel: {
    '': notifyRenderer<TunnelState>(),
    connect: invoke<void, void>(),
    disconnect: invoke<void, void>(),
    reconnect: invoke<void, void>(),
  },
  settings: {
    '': notifyRenderer<ISettings>(),
    importFile: invoke<string, void>(),
    importText: invoke<string, void>(),
    apiAccessMethodSettingChange: notifyRenderer<AccessMethodSetting>(),
    setAllowLan: invoke<boolean, void>(),
    setShowBetaReleases: invoke<boolean, void>(),
    setEnableIpv6: invoke<boolean, void>(),
    setBlockWhenDisconnected: invoke<boolean, void>(),
    setBridgeState: invoke<BridgeState, void>(),
    setOpenVpnMssfix: invoke<number | undefined, void>(),
    setWireguardMtu: invoke<number | undefined, void>(),
    setWireguardQuantumResistant: invoke<boolean | undefined, void>(),
    setRelaySettings: invoke<RelaySettings, void>(),
    updateBridgeSettings: invoke<BridgeSettings, void>(),
    setDnsOptions: invoke<IDnsOptions, void>(),
    setObfuscationSettings: invoke<ObfuscationSettings, void>(),
    addApiAccessMethod: invoke<NewAccessMethodSetting, string>(),
    updateApiAccessMethod: invoke<AccessMethodSetting, void>(),
    removeApiAccessMethod: invoke<string, void>(),
    setApiAccessMethod: invoke<string, void>(),
    testApiAccessMethodById: invoke<string, boolean>(),
    testCustomApiAccessMethod: invoke<CustomProxy, boolean>(),
    clearAllRelayOverrides: invoke<void, void>(),
    setEnableDaita: invoke<boolean, void>(),
    setDaitaDirectOnly: invoke<boolean, void>(),
  },
  guiSettings: {
    '': notifyRenderer<IGuiSettingsState>(),
    setEnableSystemNotifications: send<boolean>(),
    setAutoConnect: send<boolean>(),
    setStartMinimized: send<boolean>(),
    setMonochromaticIcon: send<boolean>(),
    setPreferredLocale: invoke<string, ITranslations>(),
    setUnpinnedWindow: send<boolean>(),
    setAnimateMap: send<boolean>(),
  },
  account: {
    '': notifyRenderer<IAccountData | undefined>(),
    device: notifyRenderer<DeviceEvent>(),
    devices: notifyRenderer<Array<IDevice>>(),
    create: invoke<void, string>(),
    login: invoke<AccountNumber, AccountDataError | undefined>(),
    logout: invoke<void, void>(),
    getWwwAuthToken: invoke<void, string>(),
    submitVoucher: invoke<string, VoucherResponse>(),
    updateData: send<void>(),
    listDevices: invoke<AccountNumber, Array<IDevice>>(),
    removeDevice: invoke<IDeviceRemoval, void>(),
  },
  accountHistory: {
    '': notifyRenderer<AccountNumber | undefined>(),
    clear: invoke<void, void>(),
  },
  autoStart: {
    '': notifyRenderer<boolean>(),
    set: invoke<boolean, void>(),
  },
  problemReport: {
    collectLogs: invoke<string | undefined, string>(),
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
  macOsSplitTunneling: {
    needFullDiskPermissions: invoke<void, boolean>(),
  },
  splitTunneling: {
    '': notifyRenderer<ISplitTunnelingApplication[]>(),
    setState: invoke<boolean, void>(),
    getApplications: invoke<
      boolean,
      { fromCache: boolean; applications: ISplitTunnelingApplication[] }
    >(),
    addApplication: invoke<ISplitTunnelingApplication | string, void>(),
    removeApplication: invoke<ISplitTunnelingApplication, void>(),
    forgetManuallyAddedApplication: invoke<ISplitTunnelingApplication, void>(),
  },
};
