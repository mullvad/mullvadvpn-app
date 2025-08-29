import { messages } from '../../../../../../../../../../shared/gettext';
import { useConnectionIsBlocked } from '../../../../../../../../../redux/hooks';
import { ResumeButton } from '../../../../../resume-button';

export function ResumeDownloadButton() {
  const { isBlocked } = useConnectionIsBlocked();

  return (
    <ResumeButton disabled={isBlocked}>
      {
        // TRANSLATORS: Button text to resume updating
        messages.pgettext('app-upgrade-view', 'Resume')
      }
    </ResumeButton>
  );
}
