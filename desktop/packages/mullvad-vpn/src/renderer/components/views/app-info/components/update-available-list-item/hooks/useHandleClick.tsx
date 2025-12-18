import { usePushAppUpgrade } from '../../../../../../history/hooks';
import { isPlatform } from '../../../../../../utils';
import { useOpenDownloadUrl } from './useOpenDownloadUrl';

export const useHandleClick = () => {
  const openDownloadUrl = useOpenDownloadUrl();
  const pushAppUpgrade = usePushAppUpgrade();
  const isLinux = isPlatform('linux');

  return isLinux ? openDownloadUrl : pushAppUpgrade;
};
