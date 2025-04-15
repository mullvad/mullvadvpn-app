import { AppUpgradeError, AppUpgradeEvent } from '../../../shared/app-upgrade';
import { AppUpgradeAction } from './actions';

export interface AppUpgradeReduxState {
  error?: AppUpgradeError;
  errorCount: number;
  event?: AppUpgradeEvent;
  state: {
    progress?: number;
  };
}

const initialState: AppUpgradeReduxState = {
  error: undefined,
  errorCount: 0,
  event: undefined,
  state: {},
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
        state:
          action.event.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS'
            ? {
                ...state.state,
                progress: action.event.progress,
              }
            : state.state,
      };
    case 'APP_UPGRADE_SET_ERROR':
      if (action.error === 'START_INSTALLER_AUTOMATIC_FAILED') {
        return {
          ...state,
          error: action.error,
        };
      }

      return {
        ...state,
        error: action.error,
        errorCount: state.errorCount + 1,
      };
    case 'APP_UPGRADE_RESET_ERROR':
      return {
        ...state,
        error: initialState.error,
      };
    case 'APP_UPGRADE_RESET':
      return {
        ...initialState,
      };
    default:
      return state;
  }
}
