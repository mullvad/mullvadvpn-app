import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowSelectLocationSelectorExitItem() {
  const { isolatedItem } = useSelectLocationViewContext();

  const showSelectLocationSelectorExitItem = isolatedItem !== 'entry';

  return showSelectLocationSelectorExitItem;
}
