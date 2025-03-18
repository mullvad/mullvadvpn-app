import { AppUpgradeEvent } from '../../../shared/app-upgrade';
import { AppUpgradeError } from '../../../shared/constants';
import { AppUpgradeAction } from './actions';

export interface AppUpgradeReduxState {
  error?: AppUpgradeError;
  event?: AppUpgradeEvent;
}

const initialState: AppUpgradeReduxState = {
  error: undefined,
  event: undefined,
};

export function appUpgradeReducer(
  state: AppUpgradeReduxState = initialState,
  action: AppUpgradeAction,
): AppUpgradeReduxState {
  switch (action.type) {
    case 'APP_UPGRADE_SET_EVENT':
      return {
        ...state,
        event: action.event,
      };
    case 'APP_UPGRADE_SET_ERROR':
      return {
        ...state,
        error: action.error,
      };
    case 'APP_UPGRADE_RESET':
      return {
        ...initialState,
      };
    default:
      return state;
  }
}
