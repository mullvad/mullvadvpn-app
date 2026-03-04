import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useHasCustomLists() {
  const { customListLocations } = useSelectLocationViewContext();

  const hasCustomLists = customListLocations.length > 0;

  return hasCustomLists;
}
