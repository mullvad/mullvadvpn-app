import React from 'react';

import { useSelectLocationViewContext } from '../../../../../SelectLocationViewContext';
import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useEffectSetSearchTerm() {
  const { setSearchTerm } = useSelectLocationViewContext();
  const {
    searching,
    textField: { value, debouncedValue },
  } = useSelectLocationSelectorItemContext();

  React.useEffect(() => {
    if (searching) {
      if (value === debouncedValue) {
        if (debouncedValue.length >= 2) {
          setSearchTerm(debouncedValue);
        } else {
          setSearchTerm('');
        }
      }
    } else {
      setSearchTerm('');
    }
  }, [searching, setSearchTerm, debouncedValue, value]);
}
