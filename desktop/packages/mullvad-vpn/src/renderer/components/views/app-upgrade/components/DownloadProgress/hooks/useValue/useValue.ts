import { useAppUpgradeEventType, useIsAppUpgradeDownloaded } from '../../../../hooks';
import { DOWNLOAD_COMPLETE_VALUE, FALLBACK_VALUE } from './constants';
import { useGetValueDownloadProgress, useGetValueError } from './hooks';

export const useValue = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const getValueDownloadProgress = useGetValueDownloadProgress();
  const getValueError = useGetValueError();
  const isAppUpgradeDownloaded = useIsAppUpgradeDownloaded();

  if (isAppUpgradeDownloaded) {
    return DOWNLOAD_COMPLETE_VALUE;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      return getValueDownloadProgress();
    case 'APP_UPGRADE_EVENT_ERROR':
      return getValueError();
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
    case 'APP_UPGRADE_EVENT_INSTALLER_READY':
      return DOWNLOAD_COMPLETE_VALUE;
    default:
      return FALLBACK_VALUE;
  }

  return 0;
};
