import React from 'react';

import { useUpdateCustomListName } from '../../../hooks';
import { useEditCustomListDialogContext } from '../EditCustomListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const updateCustomListName = useUpdateCustomListName();
  const { customList, onLoadingChange } = useEditCustomListDialogContext();

  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useEditCustomListDialogContext();

  const updateCustomList = React.useCallback(
    async (name: string) => {
      onLoadingChange?.(true);
      const { success } = await updateCustomListName(customList.details.customList, name);
      if (success) {
        onOpenChange?.(false);
        reset(name);
      } else {
        setError(true);
      }
      onLoadingChange?.(false);
    },
    [
      updateCustomListName,
      customList.details.customList,
      onLoadingChange,
      onOpenChange,
      reset,
      setError,
    ],
  );

  return React.useCallback(
    async (event: React.FormEvent) => {
      event.preventDefault();
      if (!invalid) {
        await updateCustomList(value);
      }
    },
    [invalid, updateCustomList, value],
  );
}
