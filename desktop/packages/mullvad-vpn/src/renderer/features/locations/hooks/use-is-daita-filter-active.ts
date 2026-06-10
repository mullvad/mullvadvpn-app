import { useDaitaEnabled } from '../../daita/hooks';
import { useMultihop } from '../../multihop/hooks';
import type { LocationType } from '../types';
import { isDaitaFilterActive } from '../utils/is-daita-filter-active';

export function useIsDaitaFilterActive(locationType: LocationType) {
  const { daitaEnabled } = useDaitaEnabled();
  const { multihop } = useMultihop();

  return isDaitaFilterActive(daitaEnabled, locationType, multihop);
}
