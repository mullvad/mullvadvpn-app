import React from 'react';

import { useHistory } from '../../../lib/history';
import { BackActionFn } from '../KeyboardNavigation';

export function useHandleKeyDown(backAction: BackActionFn | undefined) {
  const { pop } = useHistory();

  return React.useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        if (event.shiftKey && window.env.development) {
          pop(true);
        } else {
          backAction?.();
        }
      }
    },
    [pop, backAction],
  );
}
