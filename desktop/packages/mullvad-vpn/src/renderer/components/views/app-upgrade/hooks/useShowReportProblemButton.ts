import { AppUpgradeError } from '../../../../../shared/daemon-rpc-types';
import { useAppUpgradeError } from './useAppUpgradeError';

export const useShowReportProblemButton = () => {
  const appUpgradeError = useAppUpgradeError();

  switch (appUpgradeError) {
    case AppUpgradeError.downloadFailed:
    case AppUpgradeError.generalError:
    case AppUpgradeError.verificationFailed:
      return true;

    default:
      return false;
  }
};
