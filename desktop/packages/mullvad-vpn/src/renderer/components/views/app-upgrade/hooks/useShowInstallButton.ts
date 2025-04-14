import {
  useAppUpgradeEventType,
  useHasAppUpgradeError,
  useShouldAppUpgradeInstallManually,
} from '../../../../hooks';
import { useAppUpgradeError } from '../../../../redux/hooks';
import { useErrorCountExceeded } from './useErrorCountExceeded';

export const useShowInstallButton = () => {
  const { error } = useAppUpgradeError();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const appUpgradeEventType = useAppUpgradeEventType();
  const errorCountExceeded = useErrorCountExceeded();
  const shouldAppUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  if (!errorCountExceeded) {
    if (!hasAppUpgradeError || error === 'START_INSTALLER_AUTOMATIC_FAILED') {
      const showInstallButton =
        shouldAppUpgradeInstallManually ||
        appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTING_INSTALLER' ||
        appUpgradeEventType === 'APP_UPGRADE_STATUS_STARTED_INSTALLER';

      return showInstallButton;
    }
  }

  return false;
};
