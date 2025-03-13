import { useIsBlocked } from '../../../hooks';

export const useShowDownloadProgress = () => {
  const isBlocked = useIsBlocked();

  const showDownloadProgress = !isBlocked;

  return showDownloadProgress;
};
