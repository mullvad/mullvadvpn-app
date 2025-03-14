import { useAppUpgradeEvent } from '../../../hooks';

export const useHasUpgradeError = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const hasUpgradeError = appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR';

  return hasUpgradeError;
};
