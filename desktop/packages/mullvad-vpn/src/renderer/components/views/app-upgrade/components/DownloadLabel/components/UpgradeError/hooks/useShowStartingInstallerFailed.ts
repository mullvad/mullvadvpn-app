import { AppUpgradeError } from '../../../../../../../../../shared/daemon-rpc-types';
import { useAppUpgradeError } from '../../../../../hooks';

export const useShowStartingInstallerFailed = () => {
  const appUpgradeError = useAppUpgradeError();

  const showStartingInstallerFailed = appUpgradeError === AppUpgradeError.startInstallerFailed;

  return showStartingInstallerFailed;
};
