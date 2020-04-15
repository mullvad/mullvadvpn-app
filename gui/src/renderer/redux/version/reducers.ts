import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  currentIsSupported: boolean;
  latest?: string;
  latestStable?: string;
  nextUpgrade?: string;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  currentIsSupported: true,
  latest: undefined,
  latestStable: undefined,
  nextUpgrade: undefined,
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
      };

    default:
      return state;
  }
}
