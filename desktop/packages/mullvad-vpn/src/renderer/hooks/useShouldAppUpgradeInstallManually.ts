import { useAppUpgradeError } from '../redux/hooks';
import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';
import { useHasAppUpgradeEvent } from './useHasAppUpgradeEvent';
import { useHasAppUpgradeVerifiedInstallerPath } from './useHasAppUpgradeVerifiedInstallerPath';

export const useShouldAppUpgradeInstallManually = () => {
  const { appUpgradeError } = useAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeVerifiedInstallerPath = useHasAppUpgradeVerifiedInstallerPath();
  const hasAppUpgradeEvent = useHasAppUpgradeEvent();

  if (hasAppUpgradeVerifiedInstallerPath) {
    if (hasAppUpgradeError) {
      if (
        appUpgradeError === 'START_INSTALLER_AUTOMATIC_FAILED' ||
        appUpgradeError === 'START_INSTALLER_FAILED'
      ) {
        return true;
      }
    } else {
      // The absence of the appUpgradeEvent means that the upgrade has been downloaded
      // and the app has been exited and restarted.
      if (!hasAppUpgradeEvent || appUpgradeEventType === 'APP_UPGRADE_STATUS_VERIFIED_INSTALLER') {
        return true;
      }
    }
  }

  return false;
};
