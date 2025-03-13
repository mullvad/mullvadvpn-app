import { useAppUpgradeEvent } from '../../../../../../../hooks';

export const useValue = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_DOWNLOAD_PROGRESS') {
    const { progress } = appUpgradeEvent;

    return progress;
  }

  return 0;
};
