import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  supported: boolean;
  isBeta: boolean;
  latestBeta?: string;
  latestStable?: string;
  suggestedUpgrade?: string;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  supported: true,
  isBeta: false,
  latestBeta: undefined,
  latestStable: undefined,
  suggestedUpgrade: undefined,
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
        supported: action.latestInfo.supported,
        latestBeta: action.latestInfo.latestBeta,
        latestStable: action.latestInfo.latestStable,
        suggestedUpgrade: action.latestInfo.suggestedUpgrade,
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
