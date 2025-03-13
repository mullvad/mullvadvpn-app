import { useAppUpgradeEvent } from '../../../../../../hooks';
import {
  DownloadProgress,
  DownloadStarted,
  ErrorMessage,
  StartingInstaller,
  VerifyingInstaller,
} from './components';

export function Label() {
  const event = useAppUpgradeEvent();

  switch (event?.type) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return <DownloadStarted />;
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      return <DownloadProgress />;
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      return <VerifyingInstaller />;
    case 'APP_UPGRADE_EVENT_STARTING_INSTALLER':
      return <StartingInstaller />;
    case 'APP_UPGRADE_EVENT_ERROR':
      return <ErrorMessage />;
    default:
      return null;
  }
}
