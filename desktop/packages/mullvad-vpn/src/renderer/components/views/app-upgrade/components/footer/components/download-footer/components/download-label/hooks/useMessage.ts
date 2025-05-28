import { messages } from '../../../../../../../../../../../shared/gettext';
import { useAppUpgradeEventType } from '../../../../../../../../../../hooks';
import { useGetDownloadProgressMessage } from './useGetDownloadProgressMessage';

export const useMessage = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const getDownloadProgressMessage = useGetDownloadProgressMessage();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_STATUS_DOWNLOAD_INITIATED':
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      // TRANSLATORS: Label displayed above a progress bar when a download is in progress
      return messages.pgettext('app-upgrade-view', 'Downloading...');
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return getDownloadProgressMessage();
    default:
      return null;
  }
};
