import styled from 'styled-components';

import { useActiveFilters } from '../../../../../features/locations/hooks/use-active-filters';
import { FlexRow } from '../../../../../lib/components/flex-row';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { DaitaFilterChip } from '../daita-filter-chip';
import { LwoFilterChip } from '../lwo-filter-chip';
import { OwnershipFilterChip } from '../ownership-filter-chip';
import { ProvidersFilterChip } from '../providers-filter-chip';
import { QuicFilterChip } from '../quic-filter-chip';

export const StyledFilterChips = styled(FlexRow)`
  // Adding a small margin to not let outline be cut off
  margin: 2px;
`;

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
    <StyledFilterChips gap="small" alignItems="center" flexWrap="wrap">
      {isOwnershipFilterActive && <OwnershipFilterChip />}
      {isProvidersFilterActive && <ProvidersFilterChip />}
      {isDaitaFilterActive && <DaitaFilterChip />}
      {isQuicFilterActive && <QuicFilterChip />}
      {isLwoFilterActive && <LwoFilterChip />}
    </StyledFilterChips>
  );
}
