import { AppUpgradeEvent, AppUpgradeStep } from '../../../../shared/app-upgrade';

export const convertEventTypeToStep = (
  appUpgradeEventType: AppUpgradeEvent['type'] | undefined,
): AppUpgradeStep => {
  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      return 'download';
    case 'APP_UPGRADE_STATUS_AUTOMATIC_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_EXITED_INSTALLER':
    case 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER':
    case 'APP_UPGRADE_STATUS_MANUAL_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
      return 'launch';
    case 'APP_UPGRADE_STATUS_ABORTED':
      return 'pause';
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return 'verify';
    default:
      return 'initial';
  }
};
