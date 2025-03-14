import { useAppUpgradeEventType } from '../../../hooks';

export const useShowReportProblemButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_ERROR':
    case 'APP_UPGRADE_EVENT_DOWNLOAD_STARTED':
      return true;

    default:
      return false;
  }
};
