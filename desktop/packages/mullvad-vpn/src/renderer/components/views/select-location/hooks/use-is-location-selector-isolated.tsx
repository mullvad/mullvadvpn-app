import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useIsLocationSelectorIsolated() {
  const { isolatedItem } = useSelectLocationViewContext();

  return isolatedItem !== undefined;
}
