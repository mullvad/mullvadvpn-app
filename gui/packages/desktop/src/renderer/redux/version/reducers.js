// @flow

import type { ReduxAction } from '../store';

export type VersionReduxState = {
  current: string,
  latest: ?string,
  latestStable: ?string,
  upToDate: boolean,
  consistent: boolean,
};

const initialState: VersionReduxState = {
  current: '',
  latest: null,
  latestStable: null,
  upToDate: true,
  consistent: true,
};

const checkIfLatest = (current: string, latest: ?string, latestStable: ?string): boolean => {
  return latest === null || latestStable === null || current === latest || current === latestStable;
};

export default function(
  state: VersionReduxState = initialState,
  action: ReduxAction,
): VersionReduxState {
  switch (action.type) {
    case 'UPDATE_LATEST': {
      const latest = action.latestInfo.latest.latest;
      const latestStable = action.latestInfo.latest.latestStable;

      return {
        ...state,
        latest,
        latestStable,
        upToDate: checkIfLatest(state.current, latest, latestStable),
      };
    }

    case 'UPDATE_VERSION':
      return {
        ...state,
        current: action.version,
        consistent: action.consistent,
        upToDate: checkIfLatest(action.version, state.latest, state.latestStable),
      };

    default:
      return state;
  }
}
