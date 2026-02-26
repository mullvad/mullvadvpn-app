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
    daitaFilterActive,
    lwoFilterActive,
    ownershipFilterActive,
    providersFilterActive,
    quicFilterActive,
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

      {ownershipFilterActive && <OwnershipFilterChip />}
      {providersFilterActive && <ProvidersFilterChip />}
      {daitaFilterActive && <DaitaFilterChip />}
      {quicFilterActive && <QuicFilterChip />}
      {lwoFilterActive && <LwoFilterChip />}
    </Flex>
  );
}
