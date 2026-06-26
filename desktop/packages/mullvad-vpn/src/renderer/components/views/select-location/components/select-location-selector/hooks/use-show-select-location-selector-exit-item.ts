import { useSelectLocationSelectorContext } from '../SelectLocationSelectorContext';

export function useShowSelectLocationSelectorExitItem() {
  const { isolatedItem } = useSelectLocationSelectorContext();

  const showSelectLocationSelectorExitItem = isolatedItem !== 'entry';

  return showSelectLocationSelectorExitItem;
}
