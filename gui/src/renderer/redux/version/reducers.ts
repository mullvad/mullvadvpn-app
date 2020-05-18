import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  currentIsSupported: boolean;
  currentIsBeta: boolean;
  latest?: string;
  latestStable?: string;
  nextUpgrade: string | null;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  currentIsSupported: true,
  currentIsBeta: false,
  latest: undefined,
  latestStable: undefined,
  nextUpgrade: null,
  consistent: true,
};

export default function (
  state: IVersionReduxState = initialState,
  action: ReduxAction,
): IVersionReduxState {
  switch (action.type) {
    case 'UPDATE_LATEST':
      return {
        ...state,
        ...action.latestInfo,
      };

    case 'UPDATE_VERSION':
      return {
        ...state,
        current: action.version,
        consistent: action.consistent,
        currentIsBeta: action.currentIsBeta,
      };

    default:
      return state;
  }
}
