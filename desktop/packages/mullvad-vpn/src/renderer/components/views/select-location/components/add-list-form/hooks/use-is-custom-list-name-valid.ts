import React from 'react';

import { messages } from '../../../../../../../shared/gettext';
import { useCustomLists } from '../../../../../../features/location/hooks';

export function useIsCustomListNameValid() {
  const { customLists } = useCustomLists();
  const existingNames = React.useMemo(
    () => new Set(customLists.map((list) => list.name)),
    [customLists],
  );
  return React.useCallback(
    (name: string): boolean | string => {
      const trimmedName = name.trim();
      if (existingNames.has(trimmedName)) {
        return messages.pgettext('select-location-view', 'List names must be unique');
      }

      const customListNameValid = trimmedName.length > 0;

      return customListNameValid;
    },
    [existingNames],
  );
}
