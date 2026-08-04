import { LocationType } from '../types';
import { useIsDaitaFilterActive } from './use-is-daita-filter-active';
import { useIsLwoFilterActive } from './use-is-lwo-filter-active';
import { useIsOwnershipFilterActive } from './use-is-ownership-filter-active';
import { useIsProvidersFilterActive } from './use-is-providers-filter-active';
import { useIsQuicFilterActive } from './use-is-quic-filter-active';

export function useActiveFilters(locationType: LocationType) {
  const isProvidersFilterActive = useIsProvidersFilterActive(locationType);
  const isOwnershipFilterActive = useIsOwnershipFilterActive(locationType);
  const isQuicFilterActive = useIsQuicFilterActive(locationType);
  const isLwoFilterActive = useIsLwoFilterActive(locationType);
  const isDaitaFilterActive = useIsDaitaFilterActive(locationType);

  const isAnyFilterActive =
    isOwnershipFilterActive ||
    isProvidersFilterActive ||
    isDaitaFilterActive ||
    isQuicFilterActive ||
    isLwoFilterActive;

  return {
    isAnyFilterActive,
    isOwnershipFilterActive,
    isProvidersFilterActive,
    isDaitaFilterActive,
    isQuicFilterActive,
    isLwoFilterActive,
  };
}
