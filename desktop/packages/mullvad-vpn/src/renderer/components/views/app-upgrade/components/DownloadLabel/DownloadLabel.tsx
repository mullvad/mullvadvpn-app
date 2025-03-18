import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../hooks';
import {
  DownloadProgress,
  DownloadStarted,
  InstallerReady,
  UpgradeError,
  VerifyingInstaller,
} from './components';

export function DownloadLabel() {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return <UpgradeError />;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return <DownloadStarted />;
    case 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS':
      return <DownloadProgress />;
    case 'APP_UPGRADE_EVENT_VERIFYING_INSTALLER':
      return <VerifyingInstaller />;
    case 'APP_UPGRADE_EVENT_INSTALLER_READY':
      return <InstallerReady />;
    default:
      return null;
  }
}
