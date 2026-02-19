import React from 'react';

import { useEditCustomListDialogContext } from '../EditCustomListDialogContext';

export function useHandleClickUpdateCustomList() {
  const { formRef } = useEditCustomListDialogContext();

  return React.useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      formRef.current?.requestSubmit();
    },
    [formRef],
  );
}
