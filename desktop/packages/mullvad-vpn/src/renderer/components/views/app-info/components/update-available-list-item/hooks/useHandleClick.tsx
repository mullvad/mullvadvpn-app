import { usePushAppUpgrade } from '../../../../../../history/hooks';
import { useIsLinux } from './useIsLinux';
import { useOpenDownloadUrl } from './useOpenDownloadUrl';

export const useHandleClick = () => {
  const openDownloadUrl = useOpenDownloadUrl();
  const pushAppUpgrade = usePushAppUpgrade();
  const isLinux = useIsLinux();
  return isLinux ? openDownloadUrl : pushAppUpgrade;
};
