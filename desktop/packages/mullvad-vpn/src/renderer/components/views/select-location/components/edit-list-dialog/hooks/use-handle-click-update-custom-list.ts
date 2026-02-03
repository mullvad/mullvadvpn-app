import React from 'react';

import { useEditListDialogContext } from '../EditListDialogContext';

export function useHandleClickUpdateCustomList() {
  const { formRef } = useEditListDialogContext();

  return React.useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      formRef.current?.requestSubmit();
    },
    [formRef],
  );
}
