import { useAppUpgradeEventType } from '../../../../../../../../hooks';
import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useShowResumeButton = () => {
  const { isBlocked } = useConnectionIsBlocked();
  const appUpgradeEventType = useAppUpgradeEventType();

  const showResumeButton = appUpgradeEventType === 'APP_UPGRADE_STATUS_ABORTED' || isBlocked;

  return showResumeButton;
};
