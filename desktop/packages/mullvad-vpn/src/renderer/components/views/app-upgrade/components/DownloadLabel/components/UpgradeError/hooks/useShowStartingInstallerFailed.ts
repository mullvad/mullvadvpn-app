import { useAppUpgradeError } from '../../../../../hooks';

export const useShowStartingInstallerFailed = () => {
  const appUpgradeError = useAppUpgradeError();

  const showStartingInstallerFailed = appUpgradeError === 'START_INSTALLER_FAILED';

  return showStartingInstallerFailed;
};
