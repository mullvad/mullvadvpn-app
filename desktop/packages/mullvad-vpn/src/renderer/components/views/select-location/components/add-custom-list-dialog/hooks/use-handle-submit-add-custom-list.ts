import React from 'react';

import { useCustomLists } from '../../../../../../features/location/hooks';
import { useCustomListListContext } from '../../custom-list-location-list/CustomListLocationListContext';
import { useAddCustomListDialogContext } from '../AddCustomListDialogContext';

export function useHandleSubmitAddCustomList() {
  const { createCustomList: contextCreateCustomList } = useCustomLists();
  const { setAddingCustomList } = useCustomListListContext();
  const {
    onOpenChange,
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useAddCustomListDialogContext();

  const submitCustomList = React.useCallback(
    async (name: string) => {
      setAddingCustomList(true);
      const result = await contextCreateCustomList(name);
      if (result) {
        setError(true);
      } else {
        reset();
        onOpenChange?.(false);
      }
      setAddingCustomList(false);
    },
    [contextCreateCustomList, onOpenChange, reset, setAddingCustomList, setError],
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
