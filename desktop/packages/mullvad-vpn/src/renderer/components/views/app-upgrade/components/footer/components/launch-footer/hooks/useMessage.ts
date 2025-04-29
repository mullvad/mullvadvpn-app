import { messages } from '../../../../../../../../../shared/gettext';
import { useDisabled } from './useDisabled';

export const useMessage = () => {
  const disabled = useDisabled();

  if (disabled) {
    // TRANSLATORS: Button text to when starting the installer for an update
    return messages.pgettext('app-upgrade-view', 'Starting installer...');
  }

  // TRANSLATORS: Button text to install an update
  return messages.pgettext('app-upgrade-view', 'Install update');
};
