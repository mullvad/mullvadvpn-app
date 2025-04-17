import {
  useHasAppUpgradeError,
  useHasAppUpgradeInitiated,
  useShouldAppUpgradeInstallManually,
} from '../../../../hooks';
import { useAppUpgradeError } from '../../../../redux/hooks';

export const useShowDownloadProgress = () => {
  const { error } = useAppUpgradeError();
  const hasAppUpgradeInitiated = useHasAppUpgradeInitiated();
  const hasAppUpgradeError = useHasAppUpgradeError();
  const shouldUpgradeInstallManually = useShouldAppUpgradeInstallManually();

  if (shouldUpgradeInstallManually) return true;

  if (hasAppUpgradeError) {
    if (
      error === 'START_INSTALLER_AUTOMATIC_FAILED' ||
      error === 'START_INSTALLER_FAILED' ||
      error === 'VERIFICATION_FAILED'
    ) {
      return true;
    }

    return false;
  }

  const showDownloadProgress = hasAppUpgradeInitiated;

  return showDownloadProgress;
};
