import { useHasAppUpgradeError } from '../../../../hooks';
import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowRetryUpgradeButton = () => {
  const { appUpgradeError } = useAppUpgradeError();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    switch (appUpgradeError) {
      case 'DOWNLOAD_FAILED':
      case 'GENERAL_ERROR':
      case 'START_INSTALLER_FAILED':
      case 'VERIFICATION_FAILED':
        return true;
      default:
        return false;
    }
  }

  return false;
};
