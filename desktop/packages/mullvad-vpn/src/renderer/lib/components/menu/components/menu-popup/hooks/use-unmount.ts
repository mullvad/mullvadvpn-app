import React from 'react';

import { useMenuContext } from '../../../MenuContext';

export const useUnmount = () => {
  const { setMounted, open } = useMenuContext();

  // We conditionally render based on mounted, which needs to be set
  // after animation finished to not cut closing animation short.
  const handleUnmount = React.useCallback(() => {
    if (!open) {
      setMounted(false);
    }
  }, [open, setMounted]);

  return handleUnmount;
};
