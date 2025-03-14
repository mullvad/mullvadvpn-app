import { useAppUpgradeEventType } from '../../../hooks';

export const useShowReportProblemButton = () => {
  const appUpgradeEventType = useAppUpgradeEventType();

  switch (appUpgradeEventType) {
    case 'APP_UPGRADE_EVENT_ERROR':
      return true;

    default:
      return false;
  }
};
