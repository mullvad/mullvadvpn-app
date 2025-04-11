import { DaemonAppUpgradeError, DaemonAppUpgradeEventStatus } from './daemon-rpc-types';

export type AppUpgradeEventStatusStartingInstaller = {
  type: 'APP_UPGRADE_STATUS_STARTING_INSTALLER';
};

export type AppUpgradeEventStatusStartedInstaller = {
  type: 'APP_UPGRADE_STATUS_STARTED_INSTALLER';
};

export type AppUpgradeEventStatusExitedInstaller = {
  type: 'APP_UPGRADE_STATUS_EXITED_INSTALLER';
};

export type AppUpgradeEventStatusDownloadInitiated = {
  type: 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED';
};

export type AppUpgradeEventStatus =
  | AppUpgradeEventStatusDownloadInitiated
  | AppUpgradeEventStatusExitedInstaller
  | AppUpgradeEventStatusStartingInstaller
  | AppUpgradeEventStatusStartedInstaller;

export type AppUpgradeEvent = DaemonAppUpgradeEventStatus | AppUpgradeEventStatus;

export type AppUpgradeError =
  | DaemonAppUpgradeError
  | 'START_INSTALLER_AUTOMATIC_FAILED'
  | 'START_INSTALLER_FAILED'
  | 'INSTALLER_FAILED';
