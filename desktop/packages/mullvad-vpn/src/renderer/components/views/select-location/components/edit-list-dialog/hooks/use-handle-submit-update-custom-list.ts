import React from 'react';

import type { ICustomList } from '../../../../../../../shared/daemon-rpc-types';
import log from '../../../../../../../shared/logging';
import { useCustomLists } from '../../../../../../features/location/hooks';
import { useEditListDialogContext } from '../EditListDialogContext';

export function useHandleSubmitUpdateCustomList() {
  const { updateCustomList: contextUpdateCustomList } = useCustomLists();
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
      const updatedList: ICustomList = { ...source.list, name };
      try {
        const result = await contextUpdateCustomList(updatedList);
        if (result) {
          setError(true);
        } else {
          onOpenChange?.(false);
          reset(name);
        }
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update list:', error.message);
      }
    },

    [contextUpdateCustomList, onOpenChange, reset, setError, source.list],
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
