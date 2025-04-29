import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../../../../../../redux/hooks';

export const useGetDownloadProgressMessage = () => {
  const { event } = useAppUpgradeEvent();

  const getDownloadProgressMessage = useCallback(() => {
    if (event?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
      const { server } = event;

      return sprintf(
        // TRANSLATORS: Label displayed above a progress bar informing the user which server
        // TRANSLATORS: the update is downloading from
        messages.pgettext('app-upgrade-view', 'Downloading from: %(server)s'),
        {
          server,
        },
      );
    }

    return null;
  }, [event]);

  return getDownloadProgressMessage;
};
