import React from 'react';

import { useCustomLists } from '../../../../../../features/location/hooks';
import { useCustomListListContext } from '../../custom-list-location-list/CustomListLocationListContext';
import { useAddCustomListFormContext } from '../AddCustomListFormContext';

export function useHandleSubmitAddCustomList() {
  const { createCustomList: contextCreateCustomList } = useCustomLists();
  const { hideAddForm, setAddingForm } = useCustomListListContext();
  const {
    form: {
      setError,
      customListTextField: { value, invalid, reset },
    },
  } = useAddCustomListFormContext();

  const submitCustomList = React.useCallback(
    async (name: string) => {
      setAddingForm(true);
      const result = await contextCreateCustomList(name);
      if (result) {
        setError(true);
      } else {
        reset();
        hideAddForm();
      }
      setAddingForm(false);
    },
    [contextCreateCustomList, hideAddForm, reset, setAddingForm, setError],
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
