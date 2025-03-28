import { messages } from '../../../../../../../../shared/gettext';
import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../../hooks';
import { useGetTextError, useGetTextTimeLeft } from './hooks';

export const useText = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const getTextTimeLeft = useGetTextTimeLeft();
  const getTextError = useGetTextError();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return getTextError();
  }

  // TODO: We must ensure that we cover all the cases where we should display a text,
  // for example we don't show 'Download complete!' when the previous install attempt
  // was canceled.
  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
      return messages.pgettext('app-upgrade-view', 'Starting download...');
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return getTextTimeLeft();
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
