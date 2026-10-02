import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useTextFieldItemContext } from '../TextFieldItemContext';

export function useHandleValueChange() {
  const {
    textField: { handleOnValueChange },
  } = useTextFieldItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleValueChange = React.useCallback(
    (id: LocationSelectorSelectedItem, value: string) => {
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
