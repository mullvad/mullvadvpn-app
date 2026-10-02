import { useSelectLocationViewContext } from '../../../SelectLocationViewContext';
import { useIsIsolated } from './use-is-isolated';

export function useIsExpanded(): boolean {
  const { isLocationSelectorExpanded } = useSelectLocationViewContext();
  const isLocationSelectorIsolated = useIsIsolated();

  if (isLocationSelectorIsolated) {
    return false;
  }

  return isLocationSelectorExpanded;
}
