// @flow

import type { ReduxAction } from '../store';

export type VersionReduxState = {
  current: string,
  consistent: boolean,
};

const initialState: VersionReduxState = {
  current: '',
  consistent: true,
};

export default function(
  state: VersionReduxState = initialState,
  action: ReduxAction,
): VersionReduxState {
  switch (action.type) {
    case 'UPDATE_VERSION':
      return {
        ...state,
        current: action.version,
        consistent: action.consistent,
      };

    default:
      return state;
  }
}
