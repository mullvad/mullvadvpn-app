import { messages } from '../../../../../../../../shared/gettext';
import { LabelTiny } from '../../../../../../../lib/components';

export function DownloadStarted() {
  return (
    <LabelTiny>
      {
        // TRANSLATORS: Label displayed above a progress bar when a download is in progress
        messages.pgettext('app-upgrade-view', 'Downloading...')
      }
    </LabelTiny>
  );
}
