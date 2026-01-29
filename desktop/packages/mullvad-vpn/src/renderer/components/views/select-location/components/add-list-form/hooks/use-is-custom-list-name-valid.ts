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
      if (existingNames.has(name.trim())) {
        return messages.pgettext('select-location-view', 'List names must be unique');
      }

      const nameIsNotEmpty = name.trim() !== '';
      if (!nameIsNotEmpty) {
        return false;
      }

      return true;
    },
    [existingNames],
  );
}
