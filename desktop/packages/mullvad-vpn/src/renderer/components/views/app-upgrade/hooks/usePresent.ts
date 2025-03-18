import { useShowDownloadLabel } from './useShowDownloadLabel';
import { useShowDownloadProgress } from './useShowDownloadProgress';

export const usePresent = () => {
  const showDownloadLabel = useShowDownloadLabel();
  const showDownloadProgress = useShowDownloadProgress();

  const present = showDownloadLabel || showDownloadProgress;

  return present;
};
