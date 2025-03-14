import { useHasUpgradeError, useIsBlocked } from '../../../hooks';

export const useShowDownloadProgress = () => {
  const hasUpgradeError = useHasUpgradeError();
  const isBlocked = useIsBlocked();

  const showDownloadProgress = !hasUpgradeError && !isBlocked;

  return showDownloadProgress;
};
