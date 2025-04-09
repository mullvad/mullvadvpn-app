import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';
import { useDisabled } from './hooks';

export function InstallButton() {
  const { appUpgradeInstallerStart } = useAppContext();
  const disabled = useDisabled();

  return (
    <Button disabled={disabled} onClick={appUpgradeInstallerStart}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to install an update
          messages.pgettext('app-upgrade-view', 'Install update')
        }
      </Button.Text>
    </Button>
  );
}
