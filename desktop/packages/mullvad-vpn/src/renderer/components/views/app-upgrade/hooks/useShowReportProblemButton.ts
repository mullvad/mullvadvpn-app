import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowReportProblemButton = () => {
  const { appUpgradeError } = useAppUpgradeError();

  switch (appUpgradeError) {
    case 'DOWNLOAD_FAILED':
    case 'GENERAL_ERROR':
    case 'VERIFICATION_FAILED':
      return true;
    default:
      return false;
  }
};
