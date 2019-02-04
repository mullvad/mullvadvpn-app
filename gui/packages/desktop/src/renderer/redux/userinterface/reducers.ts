import { ReduxAction } from '../store';

export interface IUserInterfaceReduxState {
  arrowPosition?: number;
  connectionInfoOpen: boolean;
}

const initialState: IUserInterfaceReduxState = {
  connectionInfoOpen: false,
};

export default function(
  state: IUserInterfaceReduxState = initialState,
  action: ReduxAction,
): IUserInterfaceReduxState {
  switch (action.type) {
    case 'UPDATE_WINDOW_ARROW_POSITION':
      return { ...state, arrowPosition: action.arrowPosition };

    case 'UPDATE_CONNECTION_INFO_OPEN':
      return { ...state, connectionInfoOpen: action.isOpen };

    default:
      return state;
  }
}
