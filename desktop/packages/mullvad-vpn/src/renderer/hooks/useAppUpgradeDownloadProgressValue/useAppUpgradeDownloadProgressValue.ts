import { useAppUpgradeEventType } from '../useAppUpgradeEventType';
import { useHasAppUpgradeError } from '../useHasAppUpgradeError';
import { useHasAppUpgradeVerifiedInstallerPath } from '../useHasAppUpgradeVerifiedInstallerPath';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from './constants';
import { useGetValueDownloadProgress, useGetValueError } from './hooks';

export const useAppUpgradeDownloadProgressValue = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const getValueDownloadProgress = useGetValueDownloadProgress();
  const getValueError = useGetValueError();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();

  if (hasAppUpgradeError) {
    return getValueError();
  }

  if (hasAppUpgradeVerifiedInstallerPath && !appUpgradeEventType) {
    return DOWNLOAD_COMPLETE_VALUE;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return getValueDownloadProgress();
    case 'APP_UPGRADE_STATUS_EXITED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return DOWNLOAD_COMPLETE_VALUE;
    default:
      return FALLBACK_VALUE;
  }
};
