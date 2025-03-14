import { AppUpgradeError } from '../../../../../../../../../../../../../shared/daemon-rpc-types';
import { useGetHasAppUpgradeError } from '../../../../../../../../../hooks';

export const useShowStartingInstallerFailed = () => {
  const getHasAppUpgradeError = useGetHasAppUpgradeError();
  const hasErrorStartInstallerFailed = getHasAppUpgradeError(AppUpgradeError.startInstallerFailed);

  const showStartingInstallerFailed = hasErrorStartInstallerFailed;

  return showStartingInstallerFailed;
};
