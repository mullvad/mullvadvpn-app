import styled from 'styled-components';

import {
  DaitaFilterChip,
  LwoFilterChip,
  QuicFilterChip,
} from '../../../../../features/locations/components';
import { useActiveFilters } from '../../../../../features/locations/hooks';
import { LocationType } from '../../../../../features/locations/types';
import { spacings } from '../../../../../lib/foundations';
import { useFilterViewContext } from '../../FilterViewContext';

const StyledSettingsFilterChips = styled.ul`
  display: flex;
  flex-wrap: wrap;
  padding: ${spacings.tiny};
  align-items: center;
  gap: ${spacings.small};
`;

export function SettingsFilterChips() {
  const { locationType } = useFilterViewContext();
  const { isDaitaFilterActive, isQuicFilterActive, isLwoFilterActive } = useActiveFilters(
    locationType === 'exit' ? LocationType.exit : LocationType.entry,
  );

  return (
    <StyledSettingsFilterChips>
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
    </StyledSettingsFilterChips>
  );
}
