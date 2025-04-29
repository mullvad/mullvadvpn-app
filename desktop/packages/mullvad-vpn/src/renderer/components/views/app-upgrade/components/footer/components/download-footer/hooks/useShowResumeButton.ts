import { useAppUpgradeEventType } from '../../../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowResumeButton = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const eventType = useAppUpgradeEventType();

  const showResumeButton = eventType === 'APP_UPGRADE_STATUS_ABORTED' || isBlocked;

  return showResumeButton;
};
