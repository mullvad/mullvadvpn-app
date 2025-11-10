import { useSelector } from '../../../redux/store';

export function useEnableSystemNotifications() {
  const enableSystemNotifications = useSelector(
    (state) => state.settings.guiSettings.enableSystemNotifications,
  );
  return enableSystemNotifications;
}
