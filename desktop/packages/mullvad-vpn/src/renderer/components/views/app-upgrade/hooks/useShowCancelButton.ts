import { useAppUpgradeEventType, useIsAppUpgradeInProgress } from '../../../../hooks';

export const useShowCancelButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();
  const isAppUpgradeInProgress = useIsAppUpgradeInProgress();

  if (appUpgradeEventType !== 'APP_UPGRADE_STATUS_STARTED_INSTALLER') {
    const showCancelButton = isAppUpgradeInProgress;

    return showCancelButton;
  }

  return false;
};
