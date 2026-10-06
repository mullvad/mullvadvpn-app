import React from 'react';

import type { LocationSelectorItemType } from '../../../../../../../../lib/components/location-selector/types';
import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useTextFieldItemContext } from '../TextFieldItemContext';

export function useHandleValueChange() {
  const {
    textField: { handleOnValueChange },
  } = useTextFieldItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleValueChange = React.useCallback(
    (id: LocationSelectorItemType, value: string) => {
      handleOnValueChange(value);
      if (value.length >= 2) {
        setIsolatedItem(id);
        setSearchTerm(value);
      } else {
        setIsolatedItem(undefined);
        setSearchTerm('');
      }
    },
    [handleOnValueChange, setSearchTerm, setIsolatedItem],
  );

  return handleValueChange;
}
