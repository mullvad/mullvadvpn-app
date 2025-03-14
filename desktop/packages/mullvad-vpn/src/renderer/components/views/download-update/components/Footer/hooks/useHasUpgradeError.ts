import { useAppUpgradeEventType } from '../../../hooks';

export const useHasUpgradeError = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  const hasUpgradeError = appUpgradeEventType === 'APP_UPGRADE_EVENT_ERROR';

  return hasUpgradeError;
};
