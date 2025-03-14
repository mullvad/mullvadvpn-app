import { AppUpgradeError } from '../../../../../shared/daemon-rpc-types';
import { AppUpgradeEvent } from '../../../../redux/download-update/actions';
import { useSelector } from '../../../../redux/store';

export const useAppUpgradeEvent = (): AppUpgradeEvent | undefined => {
  const appUpgradeEvent = useSelector((state) => state.appUpgrade.event);

  const event: typeof appUpgradeEvent = {
    type: 'APP_UPGRADE_EVENT_ERROR',
    error: AppUpgradeError.verificationFailed,
  };

  return event;
};
