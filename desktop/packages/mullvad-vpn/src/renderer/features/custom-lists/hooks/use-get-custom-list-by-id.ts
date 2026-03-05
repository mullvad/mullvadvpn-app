import React from 'react';

import log from '../../../../shared/logging';
import { useSelector } from '../../../redux/store';

export function useGetCustomListById() {
  const customLists = useSelector((state) => state.settings.customLists);

  return React.useCallback(
    (listId: string) => {
      const customList = customLists.find((list) => list.id === listId);
      if (customList === undefined) {
        log.error(`Failed to get custom list with id ${listId}`);
        return;
      }
      return customList;
    },
    [customLists],
  );
}
