import { useAppUpgradeEventType } from '../../../../../../../../../../../../hooks';

export const useDisabled = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  const disabled = appUpgradeEventType === 'APP_UPGRADE_STATUS_MANUAL_STARTING_INSTALLER';

  return disabled;
};
