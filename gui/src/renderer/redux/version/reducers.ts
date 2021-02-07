import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  supported: boolean;
  isBeta: boolean;
  suggestedUpgrade?: string;
  suggestedIsBeta?: boolean;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  supported: true,
  isBeta: false,
  suggestedUpgrade: undefined,
  suggestedIsBeta: false,
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
        suggestedUpgrade: action.latestInfo.suggestedUpgrade,
        suggestedIsBeta: action.latestInfo.suggestedIsBeta,
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
