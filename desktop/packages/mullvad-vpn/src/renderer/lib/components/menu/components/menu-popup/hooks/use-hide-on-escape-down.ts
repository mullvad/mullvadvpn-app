import React from 'react';

import { useMenuContext } from '../../../MenuContext';

export function useHideOnEscapeDown() {
  const { onOpenChange } = useMenuContext();

  return React.useCallback(
    (e: React.KeyboardEvent<HTMLDivElement>) => {
      if (e.key === 'Escape') {
        onOpenChange?.(false);
      }
    },
    [onOpenChange],
  );
}
