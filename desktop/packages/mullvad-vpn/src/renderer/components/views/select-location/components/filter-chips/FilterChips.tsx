import { useActiveFilters } from '../../../../../features/locations/hooks/use-active-filters';
import { FlexRow } from '../../../../../lib/components/flex-row';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { DaitaFilterChip } from '../daita-filter-chip';
import { LwoFilterChip } from '../lwo-filter-chip';
import { OwnershipFilterChip } from '../ownership-filter-chip';
import { ProvidersFilterChip } from '../providers-filter-chip';
import { QuicFilterChip } from '../quic-filter-chip';

export function FilterChips() {
  const { locationType } = useSelectLocationViewContext();
  const {
    isOwnershipFilterActive,
    isProvidersFilterActive,
    isDaitaFilterActive,
    isLwoFilterActive,
    isQuicFilterActive,
  } = useActiveFilters(locationType);

  return (
    <FlexRow gap="small" alignItems="center" flexWrap="wrap">
      {isOwnershipFilterActive && <OwnershipFilterChip />}
      {isProvidersFilterActive && <ProvidersFilterChip />}
      {isDaitaFilterActive && <DaitaFilterChip />}
      {isQuicFilterActive && <QuicFilterChip />}
      {isLwoFilterActive && <LwoFilterChip />}
    </FlexRow>
  );
}
