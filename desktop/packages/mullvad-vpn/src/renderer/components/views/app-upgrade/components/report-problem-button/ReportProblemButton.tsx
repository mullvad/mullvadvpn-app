import { messages } from '../../../../../../shared/gettext';
import { usePushProblemReport } from '../../../../../history/hooks';
import { Button } from '../../../../../lib/components';

export function ReportProblemButton() {
  const pushProblemReport = usePushProblemReport({
    state: {
      options: [{ type: 'suppress-outdated-version-warning' }],
    },
  });

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
