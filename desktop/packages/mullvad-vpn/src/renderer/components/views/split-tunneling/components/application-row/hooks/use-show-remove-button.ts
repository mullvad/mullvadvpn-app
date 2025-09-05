import { useApplicationRowContext } from '../ApplicationRowContext';

export function useShowRemoveButton() {
  const { onRemove } = useApplicationRowContext();

  const showRemoveButton = onRemove !== undefined;

  return showRemoveButton;
}
