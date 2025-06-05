import { usePushAppUpgrade } from '../../../../../../history/hooks';
import { useIsPlatformLinux } from '../../../../../../hooks';
import { useOpenDownloadUrl } from './useOpenDownloadUrl';

export const useHandleClick = () => {
  const openDownloadUrl = useOpenDownloadUrl();
  const pushAppUpgrade = usePushAppUpgrade();
  const isLinux = useIsPlatformLinux();
  return isLinux ? openDownloadUrl : pushAppUpgrade;
};
