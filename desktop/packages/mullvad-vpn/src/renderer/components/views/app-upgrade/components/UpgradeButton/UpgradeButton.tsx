import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useText } from './hooks';

export function UpgradeButton() {
  const { appUpgrade } = useAppContext();
  const text = useText();

  return (
    <Button onClick={appUpgrade}>
      <Button.Text>{text}</Button.Text>
    </Button>
  );
}
