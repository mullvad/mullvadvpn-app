import { useErrorCountExceeded } from '../../../../../hooks';

export const useShowManualDownloadLink = () => {
  const errorCountExceeded = useErrorCountExceeded();

  const showManualDownloadLink = errorCountExceeded;

  return showManualDownloadLink;
};
