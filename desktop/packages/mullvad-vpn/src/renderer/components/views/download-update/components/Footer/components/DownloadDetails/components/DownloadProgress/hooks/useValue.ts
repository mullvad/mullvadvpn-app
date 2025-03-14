import { AppUpgradeError } from '../../../../../../../../../../../shared/daemon-rpc-types';
import {
  useAppUpgradeEvent,
  useAppUpgradeEventType,
  useIsAppUpgradeDownloaded,
} from '../../../../../../../hooks';

const DOWNLOAD_COMPLETE_VALUE = 100;
const FALLBACK_VALUE = 0;

const useGetValueDownloadProgress = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getValueDownloadProgress = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS') {
      const { progress } = appUpgradeEvent;

      return progress;
    }

    return FALLBACK_VALUE;
  };

  return getValueDownloadProgress;
};

const useGetValueError = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getValueError = () => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR') {
      if (
        appUpgradeEvent.error === AppUpgradeError.startInstallerFailed ||
        appUpgradeEvent.error === AppUpgradeError.verificationFailed
      )
        return DOWNLOAD_COMPLETE_VALUE;
    }

    return FALLBACK_VALUE;
  };

  return getValueError;
};

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
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
      return DOWNLOAD_COMPLETE_VALUE;
    default:
      return FALLBACK_VALUE;
  }

  return 0;
};
