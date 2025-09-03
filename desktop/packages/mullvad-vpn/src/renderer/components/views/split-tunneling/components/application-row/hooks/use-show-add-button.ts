import { useApplicationRowContext } from '../ApplicationRowContext';

export function useShowAddButton() {
  const { onAdd } = useApplicationRowContext();

  const showAddButton = onAdd !== undefined;

  return showAddButton;
}
