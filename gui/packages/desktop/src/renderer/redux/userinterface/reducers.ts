import { ReduxAction } from '../store';

export type UserInterfaceReduxState = {
  arrowPosition?: number;
  connectionInfoOpen: boolean;
};

const initialState: UserInterfaceReduxState = {
  connectionInfoOpen: false,
};

export default function(
  state: UserInterfaceReduxState = initialState,
  action: ReduxAction,
): UserInterfaceReduxState {
  switch (action.type) {
    case 'UPDATE_WINDOW_ARROW_POSITION':
      return { ...state, arrowPosition: action.arrowPosition };

    case 'UPDATE_CONNECTION_INFO_OPEN':
      return { ...state, connectionInfoOpen: action.isOpen };

    default:
      return state;
  }
}
