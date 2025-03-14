import { useAppUpgradeEvent } from '../../../hooks';

export const useShowReportProblemButton = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  switch (appUpgradeEvent?.type) {
    case 'APP_UPGRADE_EVENT_ERROR':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return true;

    default:
      return false;
  }
};
