import React from 'react';

import { useCreateCustomList } from '../../../hooks';
import { useCreateCustomListDialogContext } from '../CreateCustomListDialogContext';

export function useHandleSubmitAddCustomList() {
  const createCustomList = useCreateCustomList();
  const { location, onLoadingChange } = useCreateCustomListDialogContext();
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
      const locationDetails = location ? [location.details] : [];
      const { success } = await createCustomList(name, locationDetails);
      if (success) {
        reset();
        onOpenChange?.(false);
      } else {
        setError(true);
      }
      onLoadingChange?.(false);
    },
    [onLoadingChange, createCustomList, location, reset, onOpenChange, setError],
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
