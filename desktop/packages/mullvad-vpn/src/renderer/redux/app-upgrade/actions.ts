import { AppUpgradeError, AppUpgradeEvent } from '../../../shared/app-upgrade';

export type AppUpgradeActionReset = {
  type: 'APP_UPGRADE_RESET';
};

export const resetAppUpgrade = (): AppUpgradeActionReset => ({
  type: 'APP_UPGRADE_RESET',
});

export type AppUpgradeActionSetError = {
  type: 'APP_UPGRADE_SET_ERROR';
  error: AppUpgradeError;
};

export const setAppUpgradeError = (error: AppUpgradeError): AppUpgradeActionSetError => ({
  type: 'APP_UPGRADE_SET_ERROR',
  error,
});

export type AppUpgradeActionSetEvent = {
  type: 'APP_UPGRADE_SET_EVENT';
  event: AppUpgradeEvent;
};

export const setAppUpgradeEvent = (event: AppUpgradeEvent): AppUpgradeActionSetEvent => ({
  type: 'APP_UPGRADE_SET_EVENT',
  event,
});

export const appUpgradeActions = {
  resetAppUpgrade,
  setAppUpgradeError,
  setAppUpgradeEvent,
};

export type AppUpgradeAction =
  | AppUpgradeActionReset
  | AppUpgradeActionSetError
  | AppUpgradeActionSetEvent;
