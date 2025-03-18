import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';

export const useShowUpgradeButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return true;
  }

  // If we don't have an event type yet it is because the user has not attempted
  // an upgrade yet.
  if (!appUpgradeEventType || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED') {
    return true;
  }

  return false;
};
