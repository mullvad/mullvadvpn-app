import { useScrollPositionContext } from '../../../ScrollPositionContext';
import { useSelectLocationSelectorContext } from '../SelectLocationSelectorContext';

export function useIsLocationSelectorExpanded(): boolean {
  const { scrollTop } = useScrollPositionContext();
  const { isolatedItem } = useSelectLocationSelectorContext();

  if (isolatedItem !== undefined) {
    return false;
  }

  if (scrollTop > 20) {
    return false;
  }

  return true;
}
