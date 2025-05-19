import { useAppUpgradeEventType } from '../../../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowResumeDownloadButton = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();

  const showResumeDownloadButton =
    appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED' || isBlocked;

  return showResumeDownloadButton;
};
