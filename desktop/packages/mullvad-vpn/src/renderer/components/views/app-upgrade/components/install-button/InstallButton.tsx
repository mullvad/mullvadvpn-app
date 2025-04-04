import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';

export function InstallButton() {
  const { appUpgradeInstallerStart } = useAppContext();

  return (
    <Button onClick={appUpgradeInstallerStart}>
      <Button.Text>
        {
          // TRANSLATORS: Button text to install an update
          messages.pgettext('app-upgrade-view', 'Install update')
        }
      </Button.Text>
    </Button>
  );
}
