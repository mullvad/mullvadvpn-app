import { MacOsScrollbarVisibility } from '../../../shared/ipc-schema';
import { IChangelog } from '../../../shared/ipc-types';
import { ReduxAction } from '../store';

export interface IUserInterfaceReduxState {
  locale: string;
  arrowPosition?: number;
  connectionPanelVisible: boolean;
  windowFocused: boolean;
  scrollPosition: Record<string, [number, number]>;
  macOsScrollbarVisibility?: MacOsScrollbarVisibility;
  connectedToDaemon: boolean;
  changelog: IChangelog;
}

const initialState: IUserInterfaceReduxState = {
  locale: 'en',
  connectionPanelVisible: false,
  windowFocused: false,
  scrollPosition: {},
  macOsScrollbarVisibility: undefined,
  connectedToDaemon: false,
  changelog: [],
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

    case 'ADD_SCROLL_POSITION':
      return {
        ...state,
        scrollPosition: { ...state.scrollPosition, [action.path]: action.scrollPosition },
      };

    case 'REMOVE_SCROLL_POSITION': {
      const scrollPosition = { ...state.scrollPosition };
      delete scrollPosition[action.path];
      return { ...state, scrollPosition };
    }

    case 'SET_MACOS_SCROLLBAR_VISIBILITY':
      return { ...state, macOsScrollbarVisibility: action.visibility };

    case 'SET_CONNECTED_TO_DAEMON':
      return { ...state, connectedToDaemon: action.connectedToDaemon };

    case 'SET_CHANGELOG':
      return {
        ...state,
        changelog: action.changelog,
      };

    default:
      return state;
  }
}
