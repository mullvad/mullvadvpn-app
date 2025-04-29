import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeInitiated } from './useHasAppUpgradeInitiated';
import { useHasAppUpgradeVerifiedInstallerPath } from './useHasAppUpgradeVerifiedInstallerPath';

export const useShouldAppUpgradeInstallManually = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();

  if (hasAppUpgradeVerifiedInstallerPath) {
    // If the app upgrade has not been initiated it means that the upgrade
    // has been downloaded and afterwards the app has been restarted.
    if (
      !hasAppUpgradeInitiated ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_EXITED_INSTALLER' ||
      appUpgradeEventType === 'APP_UPGRADE_STATUS_MANUAL_START_INSTALLER'
    ) {
      return true;
    }
  }

  return false;
};
