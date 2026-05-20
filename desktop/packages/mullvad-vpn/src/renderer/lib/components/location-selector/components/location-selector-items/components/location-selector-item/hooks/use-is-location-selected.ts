import { useLocationSelectorItemContext } from '../LocationSelectorItemContext';

export function useIsLocationSelected(id: string) {
  const { selectedItem } = useLocationSelectorItemContext();
  const selected = selectedItem === id;
  return selected;
}
