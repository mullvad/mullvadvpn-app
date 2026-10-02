import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useShowExitItem() {
  const { isolatedItem } = useSelectLocationViewContext();

  return isolatedItem !== 'entry';
}
