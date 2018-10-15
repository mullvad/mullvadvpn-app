// @flow

import type { ReduxAction } from '../store';

export type VersionReduxState = {
  current: string,
  currentIsSupported: boolean,
  latest: ?string,
  latestStable: ?string,
  nextUpgrade: ?string,
  upToDate: boolean,
  consistent: boolean,
};

const initialState: VersionReduxState = {
  current: '',
  currentIsSupported: true,
  latest: null,
  latestStable: null,
  nextUpgrade: null,
  upToDate: true,
  consistent: true,
};

export default function(
  state: VersionReduxState = initialState,
  action: ReduxAction,
): VersionReduxState {
  switch (action.type) {
    case 'UPDATE_LATEST': {
      const { latest, ...other } = action.latestInfo;
      return {
        ...state,
        ...other,
        ...latest,
      };
    }

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
