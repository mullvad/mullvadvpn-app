import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useHasAppUpgradeInitiated,
} from '../../../../hooks';
import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowUpgradeButton = () => {
  const { appUpgradeError } = useAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();

  if (hasAppUpgradeError) {
    switch (appUpgradeError) {
      case 'DOWNLOAD_FAILED':
      case 'GENERAL_ERROR':
      case 'VERIFICATION_FAILED':
        return true;
      default:
        return false;
    }
  }

  // If we don't have an event type yet it is because the user has not attempted
  // an upgrade yet.
  if (!hasAppUpgradeInitiated || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED') {
    return true;
  }

  return false;
};
