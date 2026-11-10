import React from 'react';

import { useSelectLocationViewContext } from '../SelectLocationViewContext';
import { useShowAutomaticEntryItem } from './use-show-automatic-entry-item';
import { useShowEntryItem } from './use-show-entry-item';

export function useEffectResetInvalidLocationType() {
  const { locationType, setLocationType, setIsolatedItem, setSearchTerm } =
    useSelectLocationViewContext();

  const showEntryItem = useShowEntryItem();
  const showAutomaticEntryItem = useShowAutomaticEntryItem();

  React.useEffect(() => {
    const automaticEntryUnavailable = locationType === 'automaticEntry' && !showAutomaticEntryItem;
    const entryUnavailable = locationType === 'entry' && !showEntryItem;

    if (automaticEntryUnavailable || entryUnavailable) {
      if (automaticEntryUnavailable && showEntryItem) {
        // If we previously showed automatic entry we can switch over the "regular" entry
        setLocationType('entry');
      } else {
        setLocationType('exit');
      }

      // Whenever we reset location type forcibly, reset search and isolation
      setIsolatedItem(undefined);
      setSearchTerm('');
    }
  }, [
    locationType,
    setIsolatedItem,
    setLocationType,
    setSearchTerm,
    showAutomaticEntryItem,
    showEntryItem,
  ]);
}
