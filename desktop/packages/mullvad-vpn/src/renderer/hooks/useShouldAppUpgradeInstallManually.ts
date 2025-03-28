import { useAppUpgradeError } from '../redux/hooks';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';
import { useHasAppUpgradeEvent } from './useHasAppUpgradeEvent';
import { useHasAppUpgradeVerifiedInstallerPath } from './useHasAppUpgradeVerifiedInstallerPath';

export const useShouldAppUpgradeInstallManually = () => {
  const { appUpgradeError } = useAppUpgradeError();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();
  const hasAppUpgradeEvent = useHasAppUpgradeEvent();

  if (!hasAppUpgradeVerifiedInstallerPath) {
    return false;
  }

  if (hasAppUpgradeError) {
    if (appUpgradeError === 'START_INSTALLER_FAILED') {
      return true;
    }

    return false;
  }

  // The absence of the appUpgradeEvent means that the upgrade has been downloaded
  // and the app has been exited and restarted.
  if (!hasAppUpgradeEvent) {
    return true;
  }

  return false;
};
