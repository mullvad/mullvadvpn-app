import { useLocationSelectorContext } from '../../../LocationSelectorContext';

export function useIsLocationSelected(id: string) {
  const { selectedItem } = useLocationSelectorContext();
  const selected = selectedItem === id;
  return selected;
}
