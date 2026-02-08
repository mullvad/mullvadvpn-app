import React from 'react';

import { useCustomLists } from '../../../../../../features/location/hooks';
import { useEditListDialogContext } from '../EditListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const { updateCustomListName: contextUpdateCustomListName } = useCustomLists();
  const { customList } = useEditListDialogContext();

  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useEditListDialogContext();

  const updateCustomList = React.useCallback(
    async (name: string) => {
      const result = await contextUpdateCustomListName(customList.details.customList, name);
      if (result) {
        setError(true);
      } else {
        onOpenChange?.(false);
        reset(name);
      }
    },

    [contextUpdateCustomListName, customList.details.customList, onOpenChange, reset, setError],
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
