import { messages } from '../../../../../../shared/gettext';
import { useAppContext } from '../../../../../context';
import { Button } from '../../../../../lib/components';

export type LaunchInstallerButtonProps = {
  children?: React.ReactNode;
  disabled?: boolean;
};

export function LaunchInstallerButton({ children, disabled }: LaunchInstallerButtonProps) {
  const { appUpgradeInstallerStart } = useAppContext();

  return (
    <Button disabled={disabled} onClick={appUpgradeInstallerStart}>
      <Button.Text>
        {children ||
          // TRANSLATORS: Button text to install an update
          messages.pgettext('app-upgrade-view', 'Install update')}
      </Button.Text>
    </Button>
  );
}
