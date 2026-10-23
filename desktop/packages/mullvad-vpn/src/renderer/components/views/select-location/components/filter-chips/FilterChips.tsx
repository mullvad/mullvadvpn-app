import { motion } from 'motion/react';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import {
  DaitaFilterChip,
  LwoFilterChip,
  QuicFilterChip,
} from '../../../../../features/locations/components';
import { useActiveFilters } from '../../../../../features/locations/hooks/use-active-filters';
import { durationTokens, spacings } from '../../../../../lib/foundations';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';
import { OwnershipFilterChip } from '../ownership-filter-chip';
import { ProvidersFilterChip } from '../providers-filter-chip';

export const StyledFilterChips = styled(motion.ul)`
  display: flex;
  flex-wrap: wrap;
  padding: ${spacings.tiny};
  align-items: center;
  gap: ${spacings.small};

  overflow: hidden;
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
      layout="position"
      aria-label={
        // TRANSLATORS: Accessibility description for the list of active filters.
        messages.pgettext('accessibility', 'Active filters')
      }
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: durationTokens.fast, ease: 'easeOut' }}>
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
