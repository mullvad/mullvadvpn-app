import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useEnableSystemNotifications() {
  const enableSystemNotifications = useSelector(
    (state) => state.settings.guiSettings.enableSystemNotifications,
  );

  const { setEnableSystemNotifications: contextSetEnableSystemNotifications } = useAppContext();

  const setEnableSystemNotifications = React.useCallback(
    (value: boolean) => {
      try {
        contextSetEnableSystemNotifications(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set enable system notifications', message);
      }
    },
    [contextSetEnableSystemNotifications],
  );

  return { enableSystemNotifications, setEnableSystemNotifications };
}
