import { Ownership } from '../../../../../shared/daemon-rpc-types';
import { useOwnership, useProviders } from '../../../../features/location/hooks';
import { useDaitaFilterActive } from './use-daita-active';
import { useLwoFilterActive } from './use-lwo-filter-active';
import { useQuicFilterActive } from './use-quic-filter-active';

export function useActiveFilters() {
  const { activeOwnership } = useOwnership();
  const { providers, activeProviders } = useProviders();

  const quicFilterActive = useQuicFilterActive();
  const lwoFilterActive = useLwoFilterActive();
  const daitaFilterActive = useDaitaFilterActive();

  const ownershipFilterActive = activeOwnership !== Ownership.any;
  const providersFilterActive = activeProviders.length !== providers.length;
  const anyFilterActive =
    ownershipFilterActive ||
    providersFilterActive ||
    daitaFilterActive ||
    quicFilterActive ||
    lwoFilterActive;

  return {
    anyFilterActive,
    ownershipFilterActive,
    providersFilterActive,
    daitaFilterActive,
    quicFilterActive,
    lwoFilterActive,
  };
}
