import { MacOsScrollbarVisibility } from '../../../shared/ipc-schema';
import { IChangelog } from '../../../shared/ipc-types';

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

export interface ISetChangelog {
  type: 'SET_CHANGELOG';
  changelog: IChangelog;
}

export interface ISetForceShowChanges {
  type: 'SET_FORCE_SHOW_CHANGES';
  forceShowChanges: boolean;
}

export interface ISetIsPerformingPostUpgrade {
  type: 'SET_IS_PERFORMING_POST_UPGRADE';
  isPerformingPostUpgrade: boolean;
}

export type UserInterfaceAction =
  | IUpdateLocaleAction
  | IUpdateWindowArrowPositionAction
  | IUpdateConnectionInfoOpenAction
  | ISetWindowFocusedAction
  | ISetMacOsScrollbarVisibility
  | ISetConnectedToDaemon
  | ISetChangelog
  | ISetForceShowChanges
  | ISetIsPerformingPostUpgrade;

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

function setChangelog(changelog: IChangelog): ISetChangelog {
  return {
    type: 'SET_CHANGELOG',
    changelog,
  };
}

function setForceShowChanges(forceShowChanges: boolean): ISetForceShowChanges {
  return {
    type: 'SET_FORCE_SHOW_CHANGES',
    forceShowChanges,
  };
}

function setIsPerformingPostUpgrade(isPerformingPostUpgrade: boolean): ISetIsPerformingPostUpgrade {
  return {
    type: 'SET_IS_PERFORMING_POST_UPGRADE',
    isPerformingPostUpgrade,
  };
}

export default {
  updateLocale,
  updateWindowArrowPosition,
  toggleConnectionPanel,
  setWindowFocused,
  setMacOsScrollbarVisibility,
  setConnectedToDaemon,
  setChangelog,
  setForceShowChanges,
  setIsPerformingPostUpgrade,
};
