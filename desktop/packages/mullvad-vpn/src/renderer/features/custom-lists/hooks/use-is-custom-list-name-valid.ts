import React from 'react';

import { messages } from '../../../../shared/gettext';
import { useSettingsCustomLists } from '../../../redux/hooks';

export function useIsCustomListNameValid() {
  const { customLists } = useSettingsCustomLists();
  const existingNames = React.useMemo(
    () => new Set(customLists.map((list) => list.name)),
    [customLists],
  );
  return React.useCallback(
    (name: string): boolean | string => {
      const trimmedName = name.trim();
      if (existingNames.has(trimmedName)) {
        return messages.pgettext('custom-list-feature', 'List names must be unique');
      }

      const customListNameValid = trimmedName.length > 0;

      return customListNameValid;
    },
    [existingNames],
  );
}
