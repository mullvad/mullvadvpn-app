import { useApplicationRowContext } from '../ApplicationRowContext';

export function useShowDeleteButton() {
  const { onDelete } = useApplicationRowContext();

  const showDeleteButton = onDelete !== undefined;

  return showDeleteButton;
}
