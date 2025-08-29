import { useAppUpgradeError } from '../../../../../../../../../redux/hooks';
import { RetryLaunchInstallerButton, RetryUpgradeButton } from './components';

export function RetryButton() {
  const { error } = useAppUpgradeError();

  switch (error) {
    case 'INSTALLER_FAILED':
    case 'START_INSTALLER_FAILED':
      return <RetryLaunchInstallerButton />;
    case 'DOWNLOAD_FAILED':
    case 'GENERAL_ERROR':
    case 'VERIFICATION_FAILED':
      return <RetryUpgradeButton />;
    default:
      return null;
  }
}
