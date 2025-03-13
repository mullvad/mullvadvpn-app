import {
  APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS,
  APP_UPGRADE_EVENT_DOWNLOAD_STARTED,
  APP_UPGRADE_EVENT_ERROR,
  APP_UPGRADE_EVENT_STARTING_INSTALLER,
  APP_UPGRADE_EVENT_VERIFYING_INSTALLER,
  AppUpgradeEvent,
} from './actions';

export interface AppUpgradeState {
  event?: AppUpgradeEvent;
}

const initialState: AppUpgradeState = {
  event: undefined,
};

export function appUpgradeReducer(
  state: AppUpgradeState = initialState,
  action: AppUpgradeEvent,
): AppUpgradeState {
  switch (action.type) {
    case APP_UPGRADE_EVENT_DOWNLOAD_STARTED:
      return {
        ...state,
        event: action,
      };
    case APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS:
      return {
        ...state,
        event: action,
      };
    case APP_UPGRADE_EVENT_VERIFYING_INSTALLER:
      return {
        ...state,
        event: action,
      };
    case APP_UPGRADE_EVENT_STARTING_INSTALLER:
      return {
        ...state,
        event: action,
      };
    case APP_UPGRADE_EVENT_ERROR:
      return {
        ...state,
        event: action,
      };
    default:
      return state;
  }
}
