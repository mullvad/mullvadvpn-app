import { messages } from '../../../../../../shared/gettext';
import { FilterChip } from '../../../../../lib/components';

export function EntryFilterChip() {
  return (
    <FilterChip>
      <FilterChip.Text>
        {
          // TRANSLATORS: Label for a filter chip explaining that the user is selecting
          // TRANSLATORS: an entry location
          messages.pgettext('select-location-view', 'Entry')
        }
      </FilterChip.Text>
    </FilterChip>
  );
}
