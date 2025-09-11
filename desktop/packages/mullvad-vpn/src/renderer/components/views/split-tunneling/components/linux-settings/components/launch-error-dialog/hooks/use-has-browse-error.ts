import { useLinuxSettingsContext } from '../../../LinuxSettingsContext';

export function useHasBrowseError() {
  const { browseError } = useLinuxSettingsContext();

  const hasBrowseError = browseError !== undefined;

  return hasBrowseError;
}
