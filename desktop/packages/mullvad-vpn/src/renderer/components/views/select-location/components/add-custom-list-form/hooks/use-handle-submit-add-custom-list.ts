import React from 'react';

import log from '../../../../../../../shared/logging';
import { useCustomLists } from '../../../../../../features/location/hooks';
import { useCustomListListContext } from '../../custom-list-location-list/CustomListLocationListContext';
import { useAddCustomListFormContext } from '../AddCustomListFormContext';

export function useHandleSubmitAddCustomList() {
  const { createCustomList: contextCreateCustomList } = useCustomLists();
  const { hideAddForm } = useCustomListListContext();
  const {
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useAddCustomListFormContext();

  const submitCustomList = React.useCallback(
    async (name: string) => {
      try {
        const result = await contextCreateCustomList(name);
        if (result) {
          setError(true);
        } else {
          reset();
          hideAddForm();
        }
      } catch (e) {
        const error = e as Error;
        log.error('Failed to create list:', error.message);
      }
    },
    [contextCreateCustomList, hideAddForm, reset, setError],
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
