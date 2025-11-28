import { MacOsScrollbarVisibility } from '../../../shared/ipc-schema';
import { DaemonStatus, IChangelog } from '../../../shared/ipc-types';
import { LocationType } from '../../components/select-location/select-location-types';
import { ReduxAction } from '../store';

export interface IUserInterfaceReduxState {
  locale: string;
  arrowPosition?: number;
  connectionPanelVisible: boolean;
  windowFocused: boolean;
  macOsScrollbarVisibility?: MacOsScrollbarVisibility;
  connectedToDaemon: boolean;
  daemonStatus?: DaemonStatus;
  daemonAllowed?: boolean;
  changelog: IChangelog;
  isPerformingPostUpgrade: boolean;
  selectLocationView: LocationType;
  isMacOs13OrNewer: boolean;
  currentRouterIp?: string;
}

const initialState: IUserInterfaceReduxState = {
  locale: 'en',
  connectionPanelVisible: false,
  windowFocused: false,
  macOsScrollbarVisibility: undefined,
  connectedToDaemon: false,
  daemonAllowed: undefined,
  changelog: [],
  isPerformingPostUpgrade: false,
  selectLocationView: LocationType.exit,
  isMacOs13OrNewer: true,
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

    case 'SET_DAEMON_STATUS':
      return {
        ...state,
        daemonStatus: action.daemonStatus,
      };

    case 'SET_DAEMON_ALLOWED':
      return { ...state, daemonAllowed: action.daemonAllowed };

    case 'SET_CHANGELOG':
      return {
        ...state,
        changelog: action.changelog,
      };

    case 'SET_IS_PERFORMING_POST_UPGRADE':
      return {
        ...state,
        isPerformingPostUpgrade: action.isPerformingPostUpgrade,
      };

    case 'SET_SELECT_LOCATION_VIEW':
      return {
        ...state,
        selectLocationView: action.selectLocationView,
      };

    case 'SET_IS_MACOS13_OR_NEWER':
      return {
        ...state,
        isMacOs13OrNewer: action.isMacOs13OrNewer,
      };

    case 'SET_CURRENT_ROUTER_IP':
      return {
        ...state,
        currentRouterIp: action.currentRouterIp,
      };

    default:
      return state;
  }
}
