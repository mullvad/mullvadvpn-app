import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useIsAppUpgradeInstallerReady,
} from '../../../../hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from './constants';
import { useGetValueDownloadProgress, useGetValueError } from './hooks';

export const useValue = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const getValueDownloadProgress = useGetValueDownloadProgress();
  const getValueError = useGetValueError();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const isAppUpgradeInstallerReady = useIsAppUpgradeInstallerReady();

  if (hasAppUpgradeError) {
    return getValueError();
  }

  if (isAppUpgradeInstallerReady) {
    return DOWNLOAD_COMPLETE_VALUE;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return getValueDownloadProgress();
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return DOWNLOAD_COMPLETE_VALUE;
    default:
      return FALLBACK_VALUE;
  }
};
