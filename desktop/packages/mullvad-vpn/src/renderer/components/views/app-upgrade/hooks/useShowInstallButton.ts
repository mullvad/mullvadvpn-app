import { useAppUpgradeEventType, useShouldAppUpgradeInstallManually } from '../../../../hooks';

export const useShowInstallButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  const showInstallButton =
    shouldAppUpgradeInstallManually ||
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTING_INSTALLER' ||
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER';

  return showInstallButton;
};
