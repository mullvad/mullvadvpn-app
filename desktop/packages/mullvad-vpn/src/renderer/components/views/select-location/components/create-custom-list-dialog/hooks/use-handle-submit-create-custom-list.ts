import React from 'react';

import { useCustomLists } from '../../../../../../features/custom-lists/hooks';
import { useCreateCustomListDialogContext } from '../CreateCustomListDialogContext';

export function useHandleSubmitAddCustomList() {
  const { createCustomList: contextCreateCustomList } = useCustomLists();
  const { onLoadingChange } = useCreateCustomListDialogContext();
  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useCreateCustomListDialogContext();

  const submitCustomList = React.useCallback(
    async (name: string) => {
      onLoadingChange?.(true);
      const result = await contextCreateCustomList(name);
      if (result) {
        setError(true);
      } else {
        reset();
        onOpenChange?.(false);
      }
      onLoadingChange?.(false);
    },
    [contextCreateCustomList, onOpenChange, reset, onLoadingChange, setError],
  );

  return React.useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (!invalid) {
        await submitCustomList(value);
      }
    },
    [invalid, submitCustomList, value],
  );
}
