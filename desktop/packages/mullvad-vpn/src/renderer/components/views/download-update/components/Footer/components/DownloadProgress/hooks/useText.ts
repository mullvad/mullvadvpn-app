import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../hooks';
import useGetTextTimeLeft from './useGetTextTimeLeft';

export const useText = () => {
  const appUpgradeEvent = useAppUpgradeEvent();
  const getTextTimeLeft = useGetTextTimeLeft();

  // TODO: We must ensure that we cover all the cases where we should display a text,
  // for example we don't show 'Download complete!' when the previous install attempt
  // was canceled.

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is starting
      return messages.pgettext('download-update-view', 'Starting download...');
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      return getTextTimeLeft();
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
      // TRANSLATORS: Status text displayed below a progress bar when the download of an update is complete
      return messages.pgettext('download-update-view', 'Download complete!');
    default:
      return null;
  }
};
