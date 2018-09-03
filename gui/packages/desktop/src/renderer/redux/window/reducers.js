// @flow

import type { ReduxAction } from '../store';

export type WindowReduxState = {
  arrowPosition?: number,
};

const initialState: WindowReduxState = {};

export default function(
  state: WindowReduxState = initialState,
  action: ReduxAction,
): WindowReduxState {
  switch (action.type) {
    case 'UPDATE_WINDOW_ARROW_POSITION':
      return { ...state, arrowPosition: action.arrowPosition };

    default:
      return state;
  }
}
