import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useUnpinnedWindow() {
  const unpinnedWindow = useSelector((state) => state.settings.guiSettings.unpinnedWindow);
  const { setUnpinnedWindow: contextSetUnpinnedWindow } = useAppContext();

  const setUnpinnedWindow = React.useCallback(
    (value: boolean) => {
      try {
        contextSetUnpinnedWindow(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set unpinned window', message);
      }
    },
    [contextSetUnpinnedWindow],
  );

  return { unpinnedWindow, setUnpinnedWindow };
}
