import { useShowDownloadLabel } from './useShowDownloadLabel';
import { useShowDownloadProgress } from './useShowDownloadProgress';

export const useShowDownloadDetails = () => {
  const showDownloadLabel = useShowDownloadLabel();
  const showDownloadProgress = useShowDownloadProgress();

  const showDownloadDetails = showDownloadLabel || showDownloadProgress;

  return showDownloadDetails;
};
