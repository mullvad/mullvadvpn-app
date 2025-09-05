import { useSettingsContext } from '../SettingsContext';

export function useShowSpinner() {
  const { loadingDiskPermissions } = useSettingsContext();

  const showSpinner = loadingDiskPermissions;

  return showSpinner;
}
