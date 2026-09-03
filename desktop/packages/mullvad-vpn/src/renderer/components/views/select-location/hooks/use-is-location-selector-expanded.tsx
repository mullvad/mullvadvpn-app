import { useScrollPositionContext } from '../ScrollPositionContext';
import { useIsLocationSelectorIsolated } from './use-is-location-selector-isolated';

export function useIsLocationSelectorExpanded(): boolean {
  const { scrollTop } = useScrollPositionContext();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  if (isLocationSelectorIsolated) {
    return false;
  }

  if (scrollTop > 30) {
    return false;
  }

  return true;
}
