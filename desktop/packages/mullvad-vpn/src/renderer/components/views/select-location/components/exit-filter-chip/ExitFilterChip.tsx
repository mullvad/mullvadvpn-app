import { messages } from '../../../../../../shared/gettext';
import { FilterChip } from '../../../../../lib/components';

export function ExitFilterChip() {
  return (
    <FilterChip>
      <FilterChip.Text>
        {
          // TRANSLATORS: Label for a filter chip explaining that the user is selecting
          // TRANSLATORS: an exit location
          messages.pgettext('select-location-view', 'Exit')
        }
      </FilterChip.Text>
    </FilterChip>
  );
}
