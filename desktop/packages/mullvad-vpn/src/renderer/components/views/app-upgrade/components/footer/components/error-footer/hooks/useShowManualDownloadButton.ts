import { useErrorCountExceeded } from '../../../../../hooks';

export const useShowManualDownloadButton = () => {
  const errorCountExceeded = useErrorCountExceeded();

  const showManualDownloadButton = errorCountExceeded;

  return showManualDownloadButton;
};
