import React from 'react';

import { useAddCustomListFormContext } from '../AddCustomListFormContext';

export function useHandleClickCreateCustomList() {
  const { formRef } = useAddCustomListFormContext();

  return React.useCallback(
    (event: React.MouseEvent) => {
      event.preventDefault();
      formRef.current?.requestSubmit();
    },
    [formRef],
  );
}
