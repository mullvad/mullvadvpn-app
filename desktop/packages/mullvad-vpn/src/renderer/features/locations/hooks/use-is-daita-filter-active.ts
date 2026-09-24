import { useDaitaEnabled } from '../../daita/hooks';
import { useMultihop } from '../../multihop/hooks';
import { LocationType } from '../types';
import { isDaitaFilterActive } from '../utils/is-daita-filter-active';

export function useIsDaitaFilterActive(locationType: LocationType) {
  const { daitaEnabled } = useDaitaEnabled();
  const { multihop } = useMultihop();

  if (locationType === LocationType.entryAutomatic) {
    return false;
  }

  return isDaitaFilterActive(daitaEnabled, locationType, multihop);
}
