import { messages } from '../../../../../../../../../../../../../../shared/gettext';
import { Button } from '../../../../../../../../../../../../../lib/components';
import { useLaunchApplication } from '../../../../hooks';

export function LaunchButton() {
  const launchApplication = useLaunchApplication();

  return (
    <Button onClick={launchApplication}>
      <Button.Text>
        {
          // TRANSLATORS: Button label for launching an application with split tunneling.
          messages.pgettext('split-tunneling-view', 'Launch')
        }
      </Button.Text>
    </Button>
  );
}
