import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useDisabled } from './hooks';

export function UpgradeButton() {
  const { appUpgrade } = useAppContext();
  const disabled = useDisabled();

  return (
    <Button disabled={disabled} onClick={appUpgrade}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to download and install an update
          messages.pgettext('app-upgrade-view', 'Download & install')
        }
      </Button.Text>
    </Button>
  );
}
