import { messages } from '../../../../../../../../shared/gettext';
import { BodySmall } from '../../../../../../../lib/components';

export function NoChangelogUpdates() {
  return (
    <BodySmall color="whiteAlpha60">
      {
        // TRANSLATORS: Text displayed when there are no updates for this platform in the next app version
        messages.pgettext(
          'app-upgrade-view',
          'No updates or changes were made in this release for this platform.',
        )
      }
    </BodySmall>
  );
}
