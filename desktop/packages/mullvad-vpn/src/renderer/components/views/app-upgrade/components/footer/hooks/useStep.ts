import { AppUpgradeStep } from '../../../../../../../shared/app-upgrade';
import { useAppUpgradeEventType, useHasAppUpgradeError } from '../../../../../../hooks';
import { convertEventTypeToStep } from '../../../../../../redux/app-upgrade/helpers';

export const useStep = (): AppUpgradeStep => {
  const eventType = useAppUpgradeEventType();
  const hasAppUpgradeError = useHasAppUpgradeError();

  if (hasAppUpgradeError) {
    return 'error';
  }

  return convertEventTypeToStep(eventType);
};
