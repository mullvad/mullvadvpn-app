import { AppUpgradeError } from '../../../../../shared/daemon-rpc-types';
import { useAppUpgradeEvent } from './useAppUpgradeEvent';

export const useGetHasAppUpgradeError = () => {
  const appUpgradeEvent = useAppUpgradeEvent();

  const getHasError = (error: AppUpgradeError) => {
    if (appUpgradeEvent?.type === 'APP_UPGRADE_EVENT_ERROR') {
      const hasError = appUpgradeEvent.error === error;

      return hasError;
    }

    return false;
  };

  return getHasError;
};
