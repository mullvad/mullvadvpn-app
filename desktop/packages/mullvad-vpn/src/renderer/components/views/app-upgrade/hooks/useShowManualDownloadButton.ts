import { useErrorCountExceeded } from './useErrorCountExceeded';

export const useShowManualDownloadButton = () => {
  const errorCountExceeded = useErrorCountExceeded();

  const showManualDownloadButton = errorCountExceeded;

  return showManualDownloadButton;
};
