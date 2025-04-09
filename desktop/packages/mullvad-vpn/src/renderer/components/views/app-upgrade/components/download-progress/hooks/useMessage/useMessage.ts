import { messages } from '../../../../../../../../shared/gettext';
import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../../redux/hooks';
import { useGetMessageError, useGetMessageTimeLeft } from './hooks';

export const useMessage = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();
  const getMessageError = useGetMessageError();
  const getMessageTimeLeft = useGetMessageTimeLeft();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (isBlocked) {
    // TRANSLATORS: Status text displayed below a progress bar when the download of an update has been paused
    return messages.pgettext('app-upgrade-view', 'Download paused');
  }

  if (hasAppUpgradeError) {
    return getMessageError();
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
      return messages.pgettext('app-upgrade-view', 'Starting download...');
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return getMessageTimeLeft();
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('app-upgrade-view', 'Download complete!');
    default:
      return null;
  }
};
