import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowReportProblemButton = () => {
  const { error } = useAppUpgradeError();

  switch (error) {
    case 'DOWNLOAD_FAILED':
    case 'GENERAL_ERROR':
    case 'INSTALLER_FAILED':
    case 'START_INSTALLER_FAILED':
    case 'VERIFICATION_FAILED':
      return true;
    default:
      return false;
  }
};
