import { messages } from '../../../../../../../../../../../../shared/gettext';
import { LaunchInstallerButton } from '../../../../../../../launch-installer-button';
import { useDisabled } from './hooks';

export function RetryLaunchInstallerButton() {
  const disabled = useDisabled();

  return (
    <LaunchInstallerButton disabled={disabled}>
      {
        // TRANSLATORS: Button text to try again
        messages.pgettext('app-upgrade-view', 'Retry')
      }
    </LaunchInstallerButton>
  );
}
