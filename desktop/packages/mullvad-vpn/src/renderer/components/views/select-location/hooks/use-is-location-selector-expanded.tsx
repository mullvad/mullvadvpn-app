import { useScrollPositionContext } from '../ScrollPositionContext';
import { useIsLocationSelectorIsolated } from './use-is-location-selector-isolated';

export function useIsLocationSelectorExpanded(): boolean {
  const { scrollTop } = useScrollPositionContext();
  const isLocationSelectorIsolated = useIsLocationSelectorIsolated();

  return !isLocationSelectorIsolated && scrollTop < 20;
}
