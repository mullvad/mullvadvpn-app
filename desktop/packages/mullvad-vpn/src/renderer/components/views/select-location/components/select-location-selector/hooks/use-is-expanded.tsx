import { useIsLocationSelectorIsolated } from '../../../hooks';
import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';

export function useIsExpanded(): boolean {
  const { isLocationSelectorExpanded } = useSelectLocationViewContext();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  if (isLocationSelectorIsolated) {
    return false;
  }

  return isLocationSelectorExpanded;
}
