import { useAppUpgradeEventType, useShouldAppUpgradeInstallManually } from '../../../../hooks';
import { useErrorCountExceeded } from './useErrorCountExceeded';

export const useShowInstallButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();
  const errorCountExceeded = useErrorCountExceeded();

  if (errorCountExceeded) {
    return false;
  }

  const showInstallButton =
    shouldAppUpgradeInstallManually ||
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTING_INSTALLER' ||
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER';

  return showInstallButton;
};
