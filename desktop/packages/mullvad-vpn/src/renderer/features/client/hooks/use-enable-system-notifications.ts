import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useEnableSystemNotifications() {
  const enableSystemNotifications = useSelector(
    (state) => state.settings.guiSettings.enableSystemNotifications,
  );

  const { setEnableSystemNotifications } = useAppContext();
  return { enableSystemNotifications, setEnableSystemNotifications };
}
