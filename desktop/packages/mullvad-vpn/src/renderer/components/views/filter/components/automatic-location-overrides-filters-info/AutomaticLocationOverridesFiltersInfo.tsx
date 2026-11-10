import { messages } from '../../../../../../shared/gettext';
import { Note } from '../../../../../lib/components/note/Note';

export function AutomaticLocationOverridesFiltersInfo() {
  return (
    <Note>
      <Note.Icon icon="info-circle" />
      <Note.Text>
        {
          // TRANSLATORS: Info text describing that filters are not applied when an automatic location is used.
          messages.pgettext(
            'filter-view',
            'Filters are overridden when using an automatic location',
          )
        }
      </Note.Text>
    </Note>
  );
}
