import { UpgradeButton } from '../../../../../../../upgrade-button';
import { useMessage } from './hooks';

export function RetryUpgradeButton() {
  const message = useMessage();

  return <UpgradeButton>{message}</UpgradeButton>;
}
