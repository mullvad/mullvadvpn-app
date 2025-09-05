import { IconButton } from '../../../../../../../lib/components';
import { useAddApplication } from './hooks';

export function AddButton() {
  const addApplication = useAddApplication();

  return (
    <IconButton variant="secondary" onClick={addApplication}>
      <IconButton.Icon icon="add-circle" />
    </IconButton>
  );
}
