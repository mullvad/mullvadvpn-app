import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useDisabled } from './hooks';

export function CancelButton() {
  const disabled = useDisabled();
  const { appUpgradeAbort } = useAppContext();

  return (
    <Button disabled={disabled} onClick={appUpgradeAbort}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to cancel the download of an update
          messages.pgettext('app-upgrade-view', 'Cancel')
        }
      </Button.Text>
    </Button>
  );
}
