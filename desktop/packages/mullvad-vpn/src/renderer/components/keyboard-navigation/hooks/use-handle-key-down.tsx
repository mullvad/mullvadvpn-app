import React from 'react';

import { useHistory } from '../../../lib/history';
import { isPlatform } from '../../../utils';
import { BackActionFn } from '../KeyboardNavigation';

export function useHandleKeyDown(backAction: BackActionFn | undefined) {
  const { pop } = useHistory();
  const isMacOS = isPlatform('darwin');

  return React.useCallback(
    (event: KeyboardEvent) => {
      const modifierKey = isMacOS ? event.metaKey : event.altKey;
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
