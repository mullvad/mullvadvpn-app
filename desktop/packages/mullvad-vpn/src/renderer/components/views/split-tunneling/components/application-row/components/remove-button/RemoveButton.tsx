import { IconButton } from '../../../../../../../lib/components';
import { useRemoveApplication } from './hooks';

export function RemoveButton() {
  const removeApplication = useRemoveApplication();

  return (
    <IconButton variant="secondary" onClick={removeApplication}>
      <IconButton.Icon icon="remove-circle" />
    </IconButton>
  );
}
