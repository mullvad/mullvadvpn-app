import { useAppUpgradeError } from '../../../../redux/hooks';
import { useErrorCountExceeded } from './useErrorCountExceeded';

export const useShowRetryUpgradeButton = () => {
  const { error } = useAppUpgradeError();
  const errorCountExceeded = useErrorCountExceeded();

  if (!errorCountExceeded) {
    switch (error) {
      case 'DOWNLOAD_FAILED':
      case 'GENERAL_ERROR':
      case 'INSTALLER_FAILED':
      case 'START_INSTALLER_FAILED':
      case 'VERIFICATION_FAILED':
        return true;
      default:
        break;
    }
  }

  return false;
};
