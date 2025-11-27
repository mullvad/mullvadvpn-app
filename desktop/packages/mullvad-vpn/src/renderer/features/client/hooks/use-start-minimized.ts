import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useStartMinimized() {
  const startMinimized = useSelector((state) => state.settings.guiSettings.startMinimized);
  const { setStartMinimized: contextStartMinimized } = useAppContext();

  const setStartMinimized = React.useCallback(
    (value: boolean) => {
      try {
        contextStartMinimized(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set start minimized', message);
      }
    },
    [contextStartMinimized],
  );

  return { startMinimized, setStartMinimized };
}
