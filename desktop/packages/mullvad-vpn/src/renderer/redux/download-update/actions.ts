import { AppUpgradeError } from '../../../shared/daemon-rpc-types';

export const APP_UPGRADE_EVENT_DOWNLOAD_STARTED = 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED';
export const APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS = 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS';
export const APP_UPGRADE_EVENT_ABORTED = 'APP_UPGRADE_EVENT_ABORTED';
export const APP_UPGRADE_EVENT_VERIFYING_INSTALLER = 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER';
export const APP_UPGRADE_EVENT_STARTING_INSTALLER = 'APP_UPGRADE_EVENT_STARTING_INSTALLER';
export const APP_UPGRADE_EVENT_ERROR = 'APP_UPGRADE_EVENT_ERROR';

export type AppUpgradeEventDownloadStarted = {
  type: typeof APP_UPGRADE_EVENT_DOWNLOAD_STARTED;
};

export type AppUpgradeEventDownloadProgress = {
  type: typeof APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS;
  progress: number;
  server: string;
  timeLeft: number;
};

export type AppUpgradeEventAborted = {
  type: typeof APP_UPGRADE_EVENT_ABORTED;
};

export type AppUpgradeEventVerifyingInstaller = {
  type: typeof APP_UPGRADE_EVENT_VERIFYING_INSTALLER;
};

export type AppUpgradeEventStartingInstaller = {
  type: typeof APP_UPGRADE_EVENT_STARTING_INSTALLER;
};

export type AppUpgradeEventError = {
  type: typeof APP_UPGRADE_EVENT_ERROR;
  error: AppUpgradeError;
};

export type AppUpgradeEvent =
  | AppUpgradeEventDownloadStarted
  | AppUpgradeEventDownloadProgress
  | AppUpgradeEventAborted
  | AppUpgradeEventVerifyingInstaller
  | AppUpgradeEventStartingInstaller
  | AppUpgradeEventError;

export type AppUpgradeEventType = AppUpgradeEvent['type'];

export const appUpgradeDownloadStarted = (): AppUpgradeEventDownloadStarted => ({
  type: APP_UPGRADE_EVENT_DOWNLOAD_STARTED,
});

export const appUpgradeDownloadProgress = (
  progress: number,
  server: string,
  timeLeft: number,
): AppUpgradeEventDownloadProgress => ({
  type: APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS,
  progress,
  server,
  timeLeft,
});

export const appUpgradeVerifyingInstaller = (): AppUpgradeEventVerifyingInstaller => ({
  type: APP_UPGRADE_EVENT_VERIFYING_INSTALLER,
});

export const appUpgradeError = (error: AppUpgradeError): AppUpgradeEventError => ({
  type: APP_UPGRADE_EVENT_ERROR,
  error,
});

export const appUpgradeAborted = (): AppUpgradeEventAborted => ({
  type: APP_UPGRADE_EVENT_ABORTED,
});

export const appUpgradeActions = {
  appUpgradeDownloadStarted,
  appUpgradeDownloadProgress,
  appUpgradeVerifyingInstaller,
  appUpgradeError,
  appUpgradeAborted,
};
