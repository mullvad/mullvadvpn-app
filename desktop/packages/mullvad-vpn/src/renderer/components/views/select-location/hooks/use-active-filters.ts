import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { useOwnership, useProviders } from '../../../../features/locations/hooks';
import { useIsDaitaFilterActive } from '../../../../features/locations/hooks/use-is-daita-active';
import { useIsLwoFilterActive } from '../../../../features/locations/hooks/use-is-lwo-filter-active';
import { useIsQuicFilterActive } from '../../../../features/locations/hooks/use-is-quic-filter-active';
import { useSelectLocationViewContext } from '../SelectLocationViewContext';

export function useActiveFilters() {
  const { activeOwnership } = useOwnership();
  const { providers, activeProviders } = useProviders();
  const { locationType } = useSelectLocationViewContext();

  const isQuicFilterActive = useIsQuicFilterActive(locationType);
  const isLwoFilterActive = useIsLwoFilterActive(locationType);
  const isDaitaFilterActive = useIsDaitaFilterActive(locationType);

  const isOwnershipFilterActive = activeOwnership !== Ownership.any;
  const isProvidersFilterActive = activeProviders.length !== providers.length;
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
