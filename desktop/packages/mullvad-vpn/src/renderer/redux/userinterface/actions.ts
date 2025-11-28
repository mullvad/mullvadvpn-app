import { MacOsScrollbarVisibility } from '../../../shared/ipc-schema';
import { DaemonStatus, IChangelog } from '../../../shared/ipc-types';
import { LocationType } from '../../components/select-location/select-location-types';

export interface IUpdateLocaleAction {
  type: 'UPDATE_LOCALE';
  locale: string;
}

export interface IUpdateWindowArrowPositionAction {
  type: 'UPDATE_WINDOW_ARROW_POSITION';
  arrowPosition: number;
}

export interface IUpdateConnectionInfoOpenAction {
  type: 'TOGGLE_CONNECTION_PANEL';
}

export interface ISetWindowFocusedAction {
  type: 'SET_WINDOW_FOCUSED';
  focused: boolean;
}

export interface ISetMacOsScrollbarVisibility {
  type: 'SET_MACOS_SCROLLBAR_VISIBILITY';
  visibility: MacOsScrollbarVisibility;
}

export interface ISetConnectedToDaemon {
  type: 'SET_CONNECTED_TO_DAEMON';
  connectedToDaemon: boolean;
}

export interface ISetDaemonStatus {
  type: 'SET_DAEMON_STATUS';
  daemonStatus: DaemonStatus;
}

export interface ISetDaemonAllowed {
  type: 'SET_DAEMON_ALLOWED';
  daemonAllowed: boolean;
}

export interface ISetChangelog {
  type: 'SET_CHANGELOG';
  changelog: IChangelog;
}

export interface ISetIsPerformingPostUpgrade {
  type: 'SET_IS_PERFORMING_POST_UPGRADE';
  isPerformingPostUpgrade: boolean;
}

export interface ISetSelectLocationView {
  type: 'SET_SELECT_LOCATION_VIEW';
  selectLocationView: LocationType;
}

export interface ISetIsMacOs13OrNewer {
  type: 'SET_IS_MACOS13_OR_NEWER';
  isMacOs13OrNewer: boolean;
}

export interface ISetCurrentRouterIp {
  type: 'SET_CURRENT_ROUTER_IP';
  currentRouterIp?: string;
}

export type UserInterfaceAction =
  | IUpdateLocaleAction
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetWindowFocusedAction
  | ISetMacOsScrollbarVisibility
  | ISetConnectedToDaemon
  | ISetDaemonStatus
  | ISetDaemonAllowed
  | ISetChangelog
  | ISetIsPerformingPostUpgrade
  | ISetSelectLocationView
  | ISetIsMacOs13OrNewer
  | ISetCurrentRouterIp;

function updateLocale(locale: string): IUpdateLocaleAction {
  return {
    type: 'UPDATE_LOCALE',
    locale,
  };
}

function updateWindowArrowPosition(arrowPosition: number): IUpdateWindowArrowPositionAction {
  return {
    type: 'UPDATE_WINDOW_ARROW_POSITION',
    arrowPosition,
  };
}

function toggleConnectionPanel(): IUpdateConnectionInfoOpenAction {
  return {
    type: 'TOGGLE_CONNECTION_PANEL',
  };
}

function setWindowFocused(focused: boolean): ISetWindowFocusedAction {
  return {
    type: 'SET_WINDOW_FOCUSED',
    focused,
  };
}

function setMacOsScrollbarVisibility(
  visibility: MacOsScrollbarVisibility,
): ISetMacOsScrollbarVisibility {
  return {
    type: 'SET_MACOS_SCROLLBAR_VISIBILITY',
    visibility,
  };
}

function setConnectedToDaemon(connectedToDaemon: boolean): ISetConnectedToDaemon {
  return {
    type: 'SET_CONNECTED_TO_DAEMON',
    connectedToDaemon,
  };
}

function setDaemonStatus(daemonStatus: DaemonStatus): ISetDaemonStatus {
  return {
    type: 'SET_DAEMON_STATUS',
    daemonStatus: daemonStatus,
  };
}

function setDaemonAllowed(daemonAllowed: boolean): ISetDaemonAllowed {
  return {
    type: 'SET_DAEMON_ALLOWED',
    daemonAllowed,
  };
}

function setChangelog(changelog: IChangelog): ISetChangelog {
  return {
    type: 'SET_CHANGELOG',
    changelog,
  };
}

function setIsPerformingPostUpgrade(isPerformingPostUpgrade: boolean): ISetIsPerformingPostUpgrade {
  return {
    type: 'SET_IS_PERFORMING_POST_UPGRADE',
    isPerformingPostUpgrade,
  };
}

function setSelectLocationView(selectLocationView: LocationType): ISetSelectLocationView {
  return {
    type: 'SET_SELECT_LOCATION_VIEW',
    selectLocationView,
  };
}

function setIsMacOs13OrNewer(isMacOs13OrNewer: boolean): ISetIsMacOs13OrNewer {
  return {
    type: 'SET_IS_MACOS13_OR_NEWER',
    isMacOs13OrNewer,
  };
}

function setCurrenRouterIp(currentRouterIp?: string): ISetCurrentRouterIp {
  return {
    type: 'SET_CURRENT_ROUTER_IP',
    currentRouterIp,
  };
}

export default {
  updateLocale,
  updateWindowArrowPosition,
  toggleConnectionPanel,
  setWindowFocused,
  setMacOsScrollbarVisibility,
  setConnectedToDaemon,
  setDaemonStatus,
  setDaemonAllowed,
  setChangelog,
  setIsPerformingPostUpgrade,
  setSelectLocationView,
  setIsMacOs13OrNewer,
  setCurrenRouterIp,
};
