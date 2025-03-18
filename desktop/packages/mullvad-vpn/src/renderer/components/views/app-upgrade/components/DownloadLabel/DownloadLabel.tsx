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
    case 'APP_UPGRADE_STATUS_DOWNLOAD_STARTED':
      return <DownloadStarted />;
    case 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS':
      return <DownloadProgress />;
    case 'APP_UPGRADE_STATUS_VERIFYING_INSTALLER':
      return <VerifyingInstaller />;
    case 'APP_UPGRADE_STATUS_STARTED_INSTALLER':
    case 'APP_UPGRADE_STATUS_STARTING_INSTALLER':
    case 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER':
      return <InstallerReady />;
    default:
      return null;
  }
}
