import { ReduxAction } from '../store';

export enum LocationScope {
  bridge = 0,
  relay,
}

export interface IUserInterfaceReduxState {
  locale: string;
  arrowPosition?: number;
  connectionPanelVisible: boolean;
  locationScope: LocationScope;
  windowFocused: boolean;
  scrollPosition: Record<string, [number, number]>;
}

const initialState: IUserInterfaceReduxState = {
  locale: 'en',
  connectionPanelVisible: false,
  locationScope: LocationScope.relay,
  windowFocused: false,
  scrollPosition: {},
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

    case 'SET_LOCATION_SCOPE':
      return { ...state, locationScope: action.scope };

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

    default:
      return state;
  }
}
