import React from 'react';

import { useCustomLists } from '../../../../../../features/location/hooks';
import { useEditCustomListDialogContext } from '../EditCustomListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const { updateCustomListName: contextUpdateCustomListName } = useCustomLists();
  const { customList } = useEditCustomListDialogContext();

  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useEditCustomListDialogContext();

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
