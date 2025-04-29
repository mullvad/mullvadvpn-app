import { DaemonAppUpgradeError, DaemonAppUpgradeEventStatus } from './daemon-rpc-types';

export type AppUpgradeEventStatusAutomaticStartingInstaller = {
  type: 'APP_UPGRADE_STATUS_AUTOMATIC_STARTING_INSTALLER';
};

export type AppUpgradeEventStatusStartedInstaller = {
  type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER';
};

export type AppUpgradeEventStatusDownloadInitiated = {
  type: 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED';
};

export type AppUpgradeEventStatusManualStartInstaller = {
  type: 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER';
};

export type AppUpgradeEventStatusManualStartingInstaller = {
  type: 'APP_UPGRADE_STATUS_MANUAL_STARTING_INSTALLER';
};

export type AppUpgradeEventStatusExitedInstaller = {
  type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER';
};

export type AppUpgradeEventStatus =
  | AppUpgradeEventStatusStartedInstaller
  | AppUpgradeEventStatusAutomaticStartingInstaller
  | AppUpgradeEventStatusDownloadInitiated
  | AppUpgradeEventStatusExitedInstaller
  | AppUpgradeEventStatusManualStartingInstaller
  | AppUpgradeEventStatusManualStartInstaller;

export type AppUpgradeEvent = DaemonAppUpgradeEventStatus | AppUpgradeEventStatus;


export type AppUpgradeError = DaemonAppUpgradeError | 'START_INSTALLER_FAILED' | 'INSTALLER_FAILED';
