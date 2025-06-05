import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useHasAppUpgradeVerifiedInstallerPath,
} from '../../../../../../../hooks';
import { convertEventTypeToStep } from '../../../../../../../redux/app-upgrade/helpers';
import { useConnectionIsBlocked } from '../../../../../../../redux/hooks';
import { translations } from './constants';
import { useGetMessageError, useGetMessageTimeLeft } from './hooks';

export const useMessage = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();
  const getMessageError = useGetMessageError();
  const getMessageTimeLeft = useGetMessageTimeLeft();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const step = convertEventTypeToStep(appUpgradeEventType);

  if (
    (step === 'initial' && hasAppUpgradeVerifiedInstallerPath) ||
    step === 'launch' ||
    step === 'verify'
  ) {
    return translations.downloadComplete;
  }

  if (isBlocked) {
    return translations.downloadPaused;
  }

  if (hasAppUpgradeError) {
    return getMessageError();
  }

  if (step === 'pause') {
    return translations.downloadPaused;
  }

  if (step === 'download') {
    if (appUpgradeEventType === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const messageTimeLeft = getMessageTimeLeft();

      return messageTimeLeft;
    }

    return translations.downloadStarting;
  }

  return null;
};
