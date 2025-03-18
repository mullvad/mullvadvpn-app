import { messages } from '../../../../../../shared/gettext';
import { Button } from '../../../../../lib/components';
import { usePushProblemReport } from './hooks';

export function ReportProblemButton() {
  const pushProblemReport = usePushProblemReport();

  return (
    <Button onClick={pushProblemReport}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to report a problem
          messages.pgettext('app-upgrade-view', 'Report a problem')
        }
      </Button.Text>
    </Button>
  );
}
