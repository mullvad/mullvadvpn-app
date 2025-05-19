import { AppUpgradeStep } from '../../../../../../../shared/app-upgrade';
import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../hooks';
import { convertEventTypeToStep } from '../../../../../../redux/app-upgrade/helpers';
import { useConnectionIsBlocked } from '../../../../../../redux/hooks';

export const useStep = (): AppUpgradeStep => {
  const { isBlocked } = useConnectionIsBlocked();
  const eventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError && !isBlocked) {
    return 'error';
  }

  return convertEventTypeToStep(eventType);
};
