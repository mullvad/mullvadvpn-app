import { usePushAppUpgrade } from '../../../../../../history/hooks';
import { useIsLinux, useOpenDownloadUrl } from './';

export const useHandleClick = () => {
  const openDownloadUrl = useOpenDownloadUrl();
  const pushAppUpgrade = usePushAppUpgrade();
  const isLinux = useIsLinux();
  return isLinux ? openDownloadUrl : pushAppUpgrade;
};
