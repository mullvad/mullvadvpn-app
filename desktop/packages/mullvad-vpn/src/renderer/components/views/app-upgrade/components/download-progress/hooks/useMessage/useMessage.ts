import { messages } from '../../../../../../../../shared/gettext';
import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useHasAppUpgradeVerifiedInstallerPath,
} from '../../../../../../../hooks';
import { convertEventTypeToStep } from '../../../../../../../redux/app-upgrade/helpers';
import { useConnectionIsBlocked } from '../../../../../../../redux/hooks';
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
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
    return messages.pgettext('app-upgrade-view', 'Download complete!');
  }

  if (step === 'pause' || isBlocked) {
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update has been paused
    return messages.pgettext('app-upgrade-view', 'Download paused');
  }

  if (hasAppUpgradeError) {
    return getMessageError();
  }

  if (step === 'download') {
    if (appUpgradeEventType === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const messageTimeLeft = getMessageTimeLeft();

      return messageTimeLeft;
    }

    // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
    return messages.pgettext('app-upgrade-view', 'Starting download...');
  }

  return null;
};
