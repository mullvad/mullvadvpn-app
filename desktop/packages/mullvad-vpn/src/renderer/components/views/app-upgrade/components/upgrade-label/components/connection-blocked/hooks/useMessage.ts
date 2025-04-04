import { messages } from '../../../../../../../../../shared/gettext';
import { useConnectionIsBlocked } from '../../../../../../../../redux/hooks';

export const useMessage = () => {
  const { isBlocked } = useConnectionIsBlocked();

  if (isBlocked) {
    // TRANSLATORS: Label displayed when an error occurred due to the connection being blocked
    return messages.pgettext(
      'app-upgrade-view',
      'Connection blocked. Try changing server or other settings',
    );
  }

  return null;
};
