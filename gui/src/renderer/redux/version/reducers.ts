import { ReduxAction } from '../store';

export interface IVersionReduxState {
  current: string;
  currentIsSupported: boolean;
  latest?: string;
  latestStable?: string;
  nextUpgrade?: string;
  upToDate: boolean;
  consistent: boolean;
}

const initialState: IVersionReduxState = {
  current: '',
  currentIsSupported: true,
  latest: undefined,
  latestStable: undefined,
  nextUpgrade: undefined,
  upToDate: true,
  consistent: true,
};

export default function(
  state: IVersionReduxState = initialState,
  action: ReduxAction,
): IVersionReduxState {
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
