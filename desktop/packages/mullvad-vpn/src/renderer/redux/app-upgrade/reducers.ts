import { AppUpgradeError } from '../../../shared/daemon-rpc-types';
import { AppUpgradeAction, AppUpgradeEvent } from './actions';

export interface AppUpgradeState {
  error?: AppUpgradeError;
  event?: AppUpgradeEvent;
}

const initialState: AppUpgradeState = {
  error: undefined,
  event: undefined,
};

export function appUpgradeReducer(
  state: AppUpgradeState = initialState,
  action: AppUpgradeAction,
): AppUpgradeState {
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
        ...state,
        ...initialState,
      };
    default:
      return state;
  }
}
