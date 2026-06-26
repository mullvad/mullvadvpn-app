import React from 'react';

import { useSelectLocationSelectorItemContext } from '../SelectLocationSelectorItemContext';
import { useHasSearchTermBeenCleared } from './use-has-search-term-been-cleared';

export function useEffectSetSearching() {
  const {
    searching,
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

  const hasSearchTermBeenCleared = useHasSearchTermBeenCleared();
  React.useEffect(() => {
    // If there no longer is a searchTerm in the `SelectLocation` view's context,
    // but there previously was one, that means that the value has been cleared
    // and the `SelectLocationSelectorItem` should no longer be in a `searching` state.
    if (searching && hasSearchTermBeenCleared) {
      setSearching(false);
    }
  }, [hasSearchTermBeenCleared, searching, setSearching]);
}
