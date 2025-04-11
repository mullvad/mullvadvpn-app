import { useAppUpgradeError } from '../redux/hooks';
import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';
import { useHasAppUpgradeInitiated } from './useHasAppUpgradeInitiated';
import { useHasAppUpgradeVerifiedInstallerPath } from './useHasAppUpgradeVerifiedInstallerPath';

export const useShouldAppUpgradeInstallManually = () => {
  const { error } = useAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();

  if (hasAppUpgradeVerifiedInstallerPath) {
    if (hasAppUpgradeError) {
      if (error === 'START_INSTALLER_AUTOMATIC_FAILED') {
        return true;
      }
    } else {
      // If the app upgrade has not been initiated it means that the upgrade
      // has been downloaded and afterwards the app has been restarted.
      if (
        !hasAppUpgradeInitiated ||
        appUpgradeEventType === 'APP_UPGRADE_STATUS_EXITED_INSTALLER' ||
        appUpgradeEventType === 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER'
      ) {
        return true;
      }
    }
  }

  return false;
};
