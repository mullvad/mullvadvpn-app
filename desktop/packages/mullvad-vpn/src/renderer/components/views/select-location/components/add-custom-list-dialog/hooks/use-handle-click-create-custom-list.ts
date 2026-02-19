import React from 'react';

import { useAddCustomListDialogContext } from '../AddCustomListDialogContext';

export function useHandleClickCreateCustomList() {
  const { formRef } = useAddCustomListDialogContext();

  return React.useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      formRef.current?.requestSubmit();
    },
    [formRef],
  );
}
