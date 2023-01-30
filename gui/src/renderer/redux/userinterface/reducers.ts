import { MacOsScrollbarVisibility } from '../../../shared/ipc-schema';
import { IChangelog } from '../../../shared/ipc-types';
import { ReduxAction } from '../store';

export interface IUserInterfaceReduxState {
  locale: string;
  arrowPosition?: number;
  connectionPanelVisible: boolean;
  windowFocused: boolean;
  macOsScrollbarVisibility?: MacOsScrollbarVisibility;
  connectedToDaemon: boolean;
  daemonAllowed?: boolean;
  changelog: IChangelog;
  forceShowChanges: boolean;
  isPerformingPostUpgrade: boolean;
}

const initialState: IUserInterfaceReduxState = {
  locale: 'en',
  connectionPanelVisible: false,
  windowFocused: false,
  macOsScrollbarVisibility: undefined,
  connectedToDaemon: false,
  daemonAllowed: undefined,
  changelog: [],
  forceShowChanges: false,
  isPerformingPostUpgrade: false,
};

export default function (
  state: IUserInterfaceReduxState = initialState,
  action: ReduxAction,
): IUserInterfaceReduxState {
  switch (action.type) {
    case 'UPDATE_LOCALE':
      return { ...state, locale: action.locale };

    case 'UPDATE_WINDOW_ARROW_POSITION':
      return { ...state, arrowPosition: action.arrowPosition };

    case 'TOGGLE_CONNECTION_PANEL':
      return { ...state, connectionPanelVisible: !state.connectionPanelVisible };

    case 'SET_WINDOW_FOCUSED':
      return { ...state, windowFocused: action.focused };

    case 'SET_MACOS_SCROLLBAR_VISIBILITY':
      return { ...state, macOsScrollbarVisibility: action.visibility };

    case 'SET_CONNECTED_TO_DAEMON':
      return { ...state, connectedToDaemon: action.connectedToDaemon };

    case 'SET_DAEMON_ALLOWED':
      return { ...state, daemonAllowed: action.daemonAllowed };

    case 'SET_CHANGELOG':
      return {
        ...state,
        changelog: action.changelog,
      };

    case 'SET_FORCE_SHOW_CHANGES':
      return {
        ...state,
        forceShowChanges: action.forceShowChanges,
      };

    case 'SET_IS_PERFORMING_POST_UPGRADE':
      return {
        ...state,
        isPerformingPostUpgrade: action.isPerformingPostUpgrade,
      };

    default:
      return state;
  }
}
