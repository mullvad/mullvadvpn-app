import React from 'react';

import { useDialogContext } from '../../../DialogContext';

export const useHandleAnimationEnd = () => {
  const { setMounted, open } = useDialogContext();

  // We conditionally render based on mounted, which needs to be set
  // after animation finished to not cut closing animation short.
  const handleAnimationEnd = React.useCallback(() => {
    if (!open) {
      setMounted(false);
    }
  }, [open, setMounted]);

  return handleAnimationEnd;
};
