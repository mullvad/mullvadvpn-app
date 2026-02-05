import React from 'react';

import { useCustomLists } from '../../../../../../features/location/hooks';
import { useEditListDialogContext } from '../EditListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const { updateCustomListName: contextUpdateCustomListName } = useCustomLists();
  const { source } = useEditListDialogContext();

  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useEditListDialogContext();

  const updateCustomList = React.useCallback(
    async (name: string) => {
      const result = await contextUpdateCustomListName(source.list.id, name);
      if (result) {
        setError(true);
      } else {
        onOpenChange?.(false);
        reset(name);
      }
    },

    [contextUpdateCustomListName, onOpenChange, reset, setError, source.list.id],
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
