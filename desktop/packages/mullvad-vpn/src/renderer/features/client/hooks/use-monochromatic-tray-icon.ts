import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useMonochromaticTrayIcon() {
  const monochromaticIcon = useSelector((state) => state.settings.guiSettings.monochromaticIcon);
  const { setMonochromaticIcon: contextSetMonochromaticIcon } = useAppContext();

  const setMonochromaticIcon = React.useCallback(
    (value: boolean) => {
      try {
        contextSetMonochromaticIcon(value);
      } catch (error) {
        const message = error instanceof Error ? error.message : '';
        log.error('Could not set monochromatic icon', message);
      }
    },
    [contextSetMonochromaticIcon],
  );

  return { monochromaticIcon, setMonochromaticIcon };
}
