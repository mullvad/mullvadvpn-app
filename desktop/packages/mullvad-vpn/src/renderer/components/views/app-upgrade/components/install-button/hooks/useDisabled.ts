import { useAppUpgradeEventType } from '../../../../../../hooks';

export const useDisabled = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  const disabled =
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTING_INSTALLER' ||
    appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER';

  return disabled;
};
