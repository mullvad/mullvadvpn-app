import { IconButton } from '../../../../../../../lib/components';
import { useDeleteApplication } from './hooks';

export function DeleteButton() {
  const deleteApplication = useDeleteApplication();

  return (
    <IconButton variant="secondary" onClick={deleteApplication}>
      <IconButton.Icon icon="cross-circle" />
    </IconButton>
  );
}
