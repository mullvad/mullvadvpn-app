import React from 'react';

import { useCustomLists } from '../../../hooks';
import { useEditCustomListDialogContext } from '../EditCustomListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const { updateCustomListName: contextUpdateCustomListName } = useCustomLists();
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
      const result = await contextUpdateCustomListName(customList.details.customList, name);
      if (result) {
        setError(true);
      } else {
        onOpenChange?.(false);
        reset(name);
      }
      onLoadingChange?.(false);
    },
    [
      contextUpdateCustomListName,
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
