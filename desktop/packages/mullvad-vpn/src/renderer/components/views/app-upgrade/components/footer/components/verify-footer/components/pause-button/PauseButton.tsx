import { messages } from '../../../../../../../../../../shared/gettext';
import { useAppContext } from '../../../../../../../../../context';
import { Button } from '../../../../../../../../../lib/components';

export function PauseButton() {
  const { appUpgradeAbort } = useAppContext();

  return (
    <Button disabled onClick={appUpgradeAbort}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to pause the download of an update
          messages.pgettext('app-upgrade-view', 'Pause')
        }
      </Button.Text>
    </Button>
  );
}
