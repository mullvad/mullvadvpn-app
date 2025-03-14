import { useAppUpgradeEvent } from '../../../hooks';

export const useShowDownloadDetails = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const showDownloadDetails = appUpgradeEvent !== undefined;

  return showDownloadDetails;
};
