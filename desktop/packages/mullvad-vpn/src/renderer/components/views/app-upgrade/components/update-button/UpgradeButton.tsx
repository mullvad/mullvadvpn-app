import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useDisabled, useMessage } from './hooks';

export function UpgradeButton() {
  const { appUpgrade } = useAppContext();
  const disabled = useDisabled();
  const message = useMessage();

  return (
    <Button disabled={disabled} onClick={appUpgrade}>
      <Button.Text>{message}</Button.Text>
    </Button>
  );
}
