import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  supported: boolean;
  isBeta: boolean;
  latest?: string;
  latestStable?: string;
  nextUpgrade: string | null;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  supported: true,
  isBeta: false,
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
        isBeta: action.isBeta,
      };

    default:
      return state;
  }
}
