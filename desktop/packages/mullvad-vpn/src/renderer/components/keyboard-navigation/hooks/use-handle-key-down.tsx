import React from 'react';

import { useIsPlatform } from '../../../hooks';
import { useHistory } from '../../../lib/history';
import { BackActionFn } from '../KeyboardNavigation';

export function useHandleKeyDown(backAction: BackActionFn | undefined) {
  const { pop } = useHistory();
  const isMacOS = useIsPlatform('darwin');

  return React.useCallback(
    (event: KeyboardEvent) => {
      const modifierKey = isMacOS ? event.metaKey : event.ctrlKey;
      if ((event.key === '[' || event.key === 'ArrowLeft') && modifierKey) {
        backAction?.();
      } else if (window.env.development) {
        if (event.key === 'h' && event.shiftKey && modifierKey) {
          pop(true);
        }
      }
    },
    [isMacOS, backAction, pop],
  );
}
