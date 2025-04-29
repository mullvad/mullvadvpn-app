import { useAppUpgradeEventType } from '../../../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowResumeDownloadButton = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const eventType = useAppUpgradeEventType();

  const showResumeDownloadButton = eventType === 'APP_UPGRADE_STATUS_ABORTED' || isBlocked;

  return showResumeDownloadButton;
};
