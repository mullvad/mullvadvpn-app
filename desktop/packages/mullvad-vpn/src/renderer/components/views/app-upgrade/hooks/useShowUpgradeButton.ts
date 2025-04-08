import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useHasAppUpgradeInitiated,
  useShouldAppUpgradeInstallManually,
} from '../../../../hooks';

export const useShowUpgradeButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  if (!hasAppUpgradeError && !shouldAppUpgradeInstallManually) {
    // If we don't have an event type yet it is because the user has not attempted
    // an upgrade yet.
    if (!hasAppUpgradeInitiated || appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED') {
      return true;
    }
  }

  return false;
};
