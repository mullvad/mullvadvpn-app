import { motion } from 'motion/react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import { useActiveFilters } from '../../../../../features/locations/hooks/use-active-filters';
import { spacings } from '../../../../../lib/foundations';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { DaitaFilterChip } from '../daita-filter-chip';
import { LwoFilterChip } from '../lwo-filter-chip';
import { OwnershipFilterChip } from '../ownership-filter-chip';
import { ProvidersFilterChip } from '../providers-filter-chip';
import { QuicFilterChip } from '../quic-filter-chip';

export const StyledFilterChips = styled(motion.ul)`
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  margin-top: ${spacings.small};
  gap: ${spacings.small};
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
    <StyledFilterChips
      layout="preserve-aspect"
      aria-label={
        // TRANSLATORS: Accessibility description for the list of active filters.
        messages.pgettext('accessibility', 'Active filters')
      }
      initial={{ y: -20, opacity: 0 }}
      animate={{ y: 0, opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25 }}>
      {isOwnershipFilterActive && (
        <li>
          <OwnershipFilterChip />
        </li>
      )}
      {isProvidersFilterActive && (
        <li>
          <ProvidersFilterChip />
        </li>
      )}
      {isDaitaFilterActive && (
        <li>
          <DaitaFilterChip />
        </li>
      )}
      {isQuicFilterActive && (
        <li>
          <QuicFilterChip />
        </li>
      )}
      {isLwoFilterActive && (
        <li>
          <LwoFilterChip />
        </li>
      )}
    </StyledFilterChips>
  );
}
