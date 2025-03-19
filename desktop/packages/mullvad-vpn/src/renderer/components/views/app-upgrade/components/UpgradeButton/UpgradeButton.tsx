import { Button } from '../../../../../lib/components';
import { useHandleOnClick, useText } from './hooks';

export function UpgradeButton() {
  const handleOnClick = useHandleOnClick();
  const text = useText();

  return (
    <Button onClick={handleOnClick}>
      <Button.Text>{text}</Button.Text>
    </Button>
  );
}
