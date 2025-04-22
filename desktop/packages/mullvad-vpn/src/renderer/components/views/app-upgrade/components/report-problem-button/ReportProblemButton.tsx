import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { useHandleClick } from './hooks';

export function ReportProblemButton() {
  const handleClick = useHandleClick();

  return (
    <Button onClick={handleClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to report a problem
          messages.pgettext('app-upgrade-view', 'Report a problem')
        }
      </Button.Text>
    </Button>
  );
}
