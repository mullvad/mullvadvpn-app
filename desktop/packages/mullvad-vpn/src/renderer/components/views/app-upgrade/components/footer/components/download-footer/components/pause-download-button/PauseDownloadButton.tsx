import { messages } from '../../../../../../../../../../shared/gettext';
import { PauseButton } from '../../../../../pause-button';

export function PauseDownloadButton() {
  return (
    <PauseButton>
      {
        // TRANSLATORS: Button text to pause the download of an update
        messages.pgettext('app-upgrade-view', 'Pause')
      }
    </PauseButton>
  );
}
