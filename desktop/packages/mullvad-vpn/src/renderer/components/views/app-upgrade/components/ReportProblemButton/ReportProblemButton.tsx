import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { useHandleOnClick } from './hooks';

export function ReportProblemButton() {
  const handleOnClick = useHandleOnClick();

  return (
    <Button onClick={handleOnClick}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to report a problem
          messages.pgettext('app-upgrade-view', 'Report a problem')
        }
      </Button.Text>
    </Button>
  );
}
