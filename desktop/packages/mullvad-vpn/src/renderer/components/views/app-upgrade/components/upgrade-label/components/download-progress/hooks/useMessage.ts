import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../../../../shared/gettext';
import { useAppUpgradeEvent } from '../../../../../../../../redux/hooks';

export const useMessage = () => {
  const { appUpgradeEvent } = useAppUpgradeEvent();

  if (appUpgradeEvent?.type === 'APP_UPGRADE_STATUS_DOWNLOAD_PROGRESS') {
    const { server } = appUpgradeEvent;

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
};
