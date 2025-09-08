import { useLinuxSettingsContext } from '../../../../../LinuxSettingsContext';
import { useApplication } from './use-application';

export function useDisabled() {
  const { splitTunnelingSupported } = useLinuxSettingsContext();
  const application = useApplication();

  const disabled =
    splitTunnelingSupported === false || application.warning === 'launches-elsewhere';

  return disabled;
}
