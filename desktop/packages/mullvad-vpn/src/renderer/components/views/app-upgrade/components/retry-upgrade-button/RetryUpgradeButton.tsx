import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useDisabled } from './hooks';

export function RetryUpgradeButton() {
  const { appUpgrade } = useAppContext();
  const disabled = useDisabled();

  return (
    <Button disabled={disabled} onClick={appUpgrade}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to retry updating
          messages.pgettext('app-upgrade-view', 'Retry')
        }
      </Button.Text>
    </Button>
  );
}
