import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useIsLocationSelectorIsolated } from './use-is-location-selector-isolated';

export function useIsLocationSelectorExpanded(): boolean {
  const { scrollTop } = useSelectLocationViewContext();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  if (isLocationSelectorIsolated) {
    return false;
  }

  if (scrollTop > 20) {
    return false;
  }

  return true;
}
