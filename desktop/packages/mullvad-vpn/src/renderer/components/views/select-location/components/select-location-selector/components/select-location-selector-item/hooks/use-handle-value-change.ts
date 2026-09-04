import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../../../../lib/components/location-selector';
import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useHandleValueChange() {
  const {
    textField: { handleOnValueChange },
  } = useSelectLocationSelectorItemContext();
  const { setSearchTerm, setIsolatedItem } = useSelectLocationViewContext();

  const handleValueChange = React.useCallback(
    (id: LocationSelectorSelectedItem, value: string) => {
      handleOnValueChange(value);

      React.startTransition(() => {
        if (value.length >= 2) {
          setSearchTerm(value);
          setIsolatedItem(id);
        } else {
          setSearchTerm('');
          setIsolatedItem(undefined);
        }
      });
    },
    [handleOnValueChange, setSearchTerm, setIsolatedItem],
  );

  return handleValueChange;
}
