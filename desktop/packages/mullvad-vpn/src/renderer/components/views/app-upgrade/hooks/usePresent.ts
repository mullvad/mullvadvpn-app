import { useShowDownloadProgress } from './useShowDownloadProgress';
import { useShowUpgradeLabel } from './useShowUpgradeLabel';

export const usePresent = () => {
  const showUpgradeLabel = useShowUpgradeLabel();
  const showDownloadProgress = useShowDownloadProgress();

  const present = showUpgradeLabel || showDownloadProgress;

  return present;
};
