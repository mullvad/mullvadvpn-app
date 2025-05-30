import { AppUpgradeError, AppUpgradeEvent } from '../../../shared/app-upgrade';

export type AppUpgradeActionReset = {
  type: 'APP_UPGRADE_RESET';
};

export const resetAppUpgrade = (): AppUpgradeActionReset => ({
  type: 'APP_UPGRADE_RESET',
});

export type AppUpgradeActionResetError = {
  type: 'APP_UPGRADE_RESET_ERROR';
};

export const resetAppUpgradeError = (): AppUpgradeActionResetError => ({
  type: 'APP_UPGRADE_RESET_ERROR',
});

export type AppUpgradeActionSetError = {
  type: 'APP_UPGRADE_SET_ERROR';
  error: AppUpgradeError;
};

export type AppUpgradeActionSetLastProgress = {
  type: 'APP_UPGRADE_SET_LAST_PROGRESS';
  lastProgress: number;
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

export const setLastProgress = (lastProgress: number): AppUpgradeActionSetLastProgress => ({
  type: 'APP_UPGRADE_SET_LAST_PROGRESS',
  lastProgress,
});

export const appUpgradeActions = {
  resetAppUpgrade,
  resetAppUpgradeError,
  setAppUpgradeError,
  setAppUpgradeEvent,
  setLastProgress,
};

export type AppUpgradeAction =
  | AppUpgradeActionReset
  | AppUpgradeActionResetError
  | AppUpgradeActionSetError
  | AppUpgradeActionSetEvent
  | AppUpgradeActionSetLastProgress;
