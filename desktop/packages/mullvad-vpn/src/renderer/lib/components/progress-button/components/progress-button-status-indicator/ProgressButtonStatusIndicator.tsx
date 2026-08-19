import { Button } from '../../../button';
import { useProgressButtonContext } from '../../ProgressButtonContext';

export function ProgressButtonStatusIndicator() {
  const { status } = useProgressButtonContext();
  if (status === 'idle') {
    return null;
  } else if (status === 'loading') {
    return <Button.Spinner />;
  } else if (status === 'success') {
    return <Button.Icon icon="checkmark" disabled={false} />;
  } else if (status === 'error') {
    return <Button.Icon icon="cross" disabled={false} />;
  }
  return status satisfies never;
}
