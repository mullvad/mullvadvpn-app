import { ReduxAction } from '../store';

export interface IUserInterfaceReduxState {
  arrowPosition?: number;
  connectionPanelVisible: boolean;
}

const initialState: IUserInterfaceReduxState = {
  connectionPanelVisible: false,
};

export default function(
  state: IUserInterfaceReduxState = initialState,
  action: ReduxAction,
): IUserInterfaceReduxState {
  switch (action.type) {
    case 'UPDATE_WINDOW_ARROW_POSITION':
      return { ...state, arrowPosition: action.arrowPosition };

    case 'TOGGLE_CONNECTION_PANEL':
      return { ...state, connectionPanelVisible: !state.connectionPanelVisible };

    default:
      return state;
  }
}
