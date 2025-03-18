import { useAppUpgradeEventType } from './useAppUpgradeEventType';
import { useHasAppUpgradeError } from './useHasAppUpgradeError';

export const useShowUpgradeButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  // If we don't have an event type yet it is because the user has not attempted
  // an upgrade yet.
  if (!appUpgradeEventType) {
    return true;
  }

  if (hasAppUpgradeError) {
    return true;
  }

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_ABORTED':
      return true;

    default:
      return false;
  }
};
