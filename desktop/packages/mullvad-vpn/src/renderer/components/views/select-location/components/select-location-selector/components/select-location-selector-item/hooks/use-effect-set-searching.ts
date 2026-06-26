import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';

export function useEffectSetSearching() {
  const {
    setSearching,
    textField: { dirty, touched },
  } = useSelectLocationSelectorItemContext();

  React.useEffect(() => {
    if (dirty) {
      setSearching(true);
    }
  }, [dirty, setSearching]);

  React.useEffect(() => {
    if (!touched) {
      setSearching(false);
    }
  }, [setSearching, touched]);
}
