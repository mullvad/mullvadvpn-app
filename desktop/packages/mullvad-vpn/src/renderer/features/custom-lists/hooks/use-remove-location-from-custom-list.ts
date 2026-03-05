import React from 'react';

import {
  compareRelayLocationGeographical,
  type RelayLocationGeographical,
} from '../../../../shared/daemon-rpc-types';
import { useGetCustomListById } from './use-get-custom-list-by-id';
import { useUpdateCustomList } from './use-update-custom-list';

export function useRemoveLocationFromCustomList() {
  const updateCustomList = useUpdateCustomList();
  const getCustomListById = useGetCustomListById();

  const removeLocationFromCustomList = React.useCallback(
    async (listId: string, location: RelayLocationGeographical) => {
      const customList = getCustomListById(listId);
      if (customList === undefined) {
        return;
      }
      const updatedCustomList = {
        ...customList,
        locations: customList.locations.filter(
          (listLocation) => !compareRelayLocationGeographical(listLocation, location),
        ),
      };

      return updateCustomList(updatedCustomList);
    },
    [getCustomListById, updateCustomList],
  );

  return removeLocationFromCustomList;
}
