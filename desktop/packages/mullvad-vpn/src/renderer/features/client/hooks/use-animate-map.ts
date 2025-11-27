import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useAnimateMap() {
  const animateMap = useSelector((state) => state.settings.guiSettings.animateMap);
  const { setAnimateMap: contextSetAnimateMap } = useAppContext();

  const setAnimateMap = React.useCallback(
    (value: boolean) => {
      try {
        contextSetAnimateMap(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set animate map', message);
      }
    },
    [contextSetAnimateMap],
  );

  return { animateMap, setAnimateMap };
}
