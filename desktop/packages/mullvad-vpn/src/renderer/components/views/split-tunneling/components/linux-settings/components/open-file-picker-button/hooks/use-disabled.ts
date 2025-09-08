import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useDisabled() {
  const { splitTunnelingSupported } = useLinuxSettingsContext();

  const disabled = splitTunnelingSupported === false;

  return disabled;
}
