import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useIsOpen() {
  const { browseError } = useLinuxSettingsContext();

  const isOpen = browseError !== undefined;

  return isOpen;
}
