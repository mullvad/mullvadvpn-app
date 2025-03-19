import { messages } from '../../../../../../../shared/gettext';
import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../hooks';
import { useGetTextError } from './useGetTextError';
import useGetTextTimeLeft from './useGetTextTimeLeft';

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
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
      return messages.pgettext('app-upgrade-view', 'Starting download...');
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      return getTextTimeLeft();
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
    case 'APP_UPGRADE_EVENT_INSTALLER_READY':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('app-upgrade-view', 'Download complete!');
    default:
      return null;
  }
};
