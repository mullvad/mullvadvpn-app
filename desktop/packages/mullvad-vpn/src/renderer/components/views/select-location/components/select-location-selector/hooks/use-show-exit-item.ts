import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useShowExitItem() {
  const { isolatedItem } = useSelectLocationViewContext();

  const showSelectLocationSelectorExitItem = isolatedItem !== 'entry';

  return showSelectLocationSelectorExitItem;
}
