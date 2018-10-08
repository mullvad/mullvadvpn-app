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

function isBeta(version: string) {
  return version.includes('-');
}

function nextUpgrade(current: string, latest: ?string, latestStable: ?string): ?string {
  if (isBeta(current)) {
    return current === latest ? null : latest;
  } else {
    return current === latestStable ? null : latestStable;
  }
}

function checkIfLatest(current: string, latest: ?string, latestStable: ?string): boolean {
  // perhaps -beta?
  if (isBeta(current)) {
    return current === latest || latest === null;
  } else {
    // must be stable
    return current === latestStable || latestStable === null;
  }
}

export default function(
  state: VersionReduxState = initialState,
  action: ReduxAction,
): VersionReduxState {
  switch (action.type) {
    case 'UPDATE_LATEST': {
      const currentIsSupported = action.latestInfo.currentIsSupported;
      const latest = action.latestInfo.latest.latest;
      const latestStable = action.latestInfo.latest.latestStable;

      return {
        ...state,
        currentIsSupported,
        latest,
        latestStable,
        nextUpgrade: nextUpgrade(state.current, latest, latestStable),
        upToDate: checkIfLatest(state.current, latest, latestStable),
      };
    }

    case 'UPDATE_VERSION':
      return {
        ...state,
        current: action.version,
        consistent: action.consistent,
        nextUpgrade: nextUpgrade(action.version, state.latest, state.latestStable),
        upToDate: checkIfLatest(action.version, state.latest, state.latestStable),
      };

    default:
      return state;
  }
}
