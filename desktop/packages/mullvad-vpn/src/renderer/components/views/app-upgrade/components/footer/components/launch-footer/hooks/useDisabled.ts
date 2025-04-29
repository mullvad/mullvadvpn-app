import { useAppUpgradeEventType } from '../../../../../../../../hooks';

export const useDisabled = () => {
  const eventType = useAppUpgradeEventType();

  const disabled =
    eventType === 'APP_UPGRADE_STATUS_AUTOMATIC_STARTING_INSTALLER' ||
    eventType === 'APP_UPGRADE_STATUS_MANUAL_STARTING_INSTALLER' ||
    eventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER';

  return disabled;
};
