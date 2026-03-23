import React from 'react';

import type { CustomListLocation } from '../types';
import { searchCustomListAndLocations } from '../utils';

export function useSearchCustomListLocations(
  customListLocations: CustomListLocation[],
  searchTerm: string,
): CustomListLocation[] {
  return React.useMemo(() => {
    if (!searchTerm) {
      return customListLocations;
    }

    return customListLocations
      .map((customList) => searchCustomListAndLocations(customList, searchTerm))
      .filter((customList) => customList !== undefined);
  }, [customListLocations, searchTerm]);
}
