import React from 'react';

import { useCreateCustomListDialogContext } from '../CreateCustomListDialogContext';

export function useHandleClickCreateCustomList() {
  const { formRef } = useCreateCustomListDialogContext();

  return React.useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      formRef.current?.requestSubmit();
    },
    [formRef],
  );
}
