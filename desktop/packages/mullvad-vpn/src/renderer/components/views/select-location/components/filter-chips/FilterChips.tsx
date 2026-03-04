import { messages } from '../../../../../../shared/gettext';
import { Flex, LabelTinySemiBold } from '../../../../../lib/components';
import { useActiveFilters } from '../../hooks/use-active-filters';
import { DaitaFilterChip } from '../daita-filter-chip';
import { LwoFilterChip } from '../lwo-filter-chip';
import { OwnershipFilterChip } from '../ownership-filter-chip';
import { ProvidersFilterChip } from '../providers-filter-chip';
import { QuicFilterChip } from '../quic-filter-chip';

export function FilterChips() {
  const {
    isOwnershipFilterActive,
    isProvidersFilterActive,
    isDaitaFilterActive,
    isLwoFilterActive,
    isQuicFilterActive,
  } = useActiveFilters();

  return (
    <Flex
      gap="small"
      alignItems="center"
      flexWrap="wrap"
      margin={{ horizontal: 'small', bottom: 'medium' }}>
      <LabelTinySemiBold>
        {messages.pgettext('select-location-view', 'Filtered:')}
      </LabelTinySemiBold>

      {isOwnershipFilterActive && <OwnershipFilterChip />}
      {isProvidersFilterActive && <ProvidersFilterChip />}
      {isDaitaFilterActive && <DaitaFilterChip />}
      {isQuicFilterActive && <QuicFilterChip />}
      {isLwoFilterActive && <LwoFilterChip />}
    </Flex>
  );
}
