import { messages } from '../../../../../../shared/gettext';
import { BodySmall } from '../../../../../lib/components';
import { Colors } from '../../../../../lib/foundations';

export function NoChangelogUpdates() {
  return (
    <BodySmall color={Colors.white60}>
      {
        // TRANSLATORS: Text displayed when there are no updates for this platform in the app version
        messages.pgettext(
          'changelog-view',
          'No updates or changes were made in this release for this platform.',
        )
      }
    </BodySmall>
  );
}
