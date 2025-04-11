import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowRetryUpgradeButton = () => {
  const { error } = useAppUpgradeError();

  switch (error) {
    case 'DOWNLOAD_FAILED':
    case 'GENERAL_ERROR':
    case 'START_INSTALLER_FAILED':
    case 'VERIFICATION_FAILED':
      return true;
    default:
      return false;
  }
};
