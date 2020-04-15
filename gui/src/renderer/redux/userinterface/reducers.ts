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
}

const initialState: IUserInterfaceReduxState = {
  locale: 'en',
  connectionPanelVisible: false,
  locationScope: LocationScope.relay,
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

    default:
      return state;
  }
}
