import React from 'react';

import { type RelayLocationGeographical } from '../../../../shared/daemon-rpc-types';
import { useGetCustomListById } from './use-get-custom-list-by-id';
import { useUpdateCustomList } from './use-update-custom-list';

export function useAddLocationToCustomList() {
  const getCustomListById = useGetCustomListById();
  const updateCustomList = useUpdateCustomList();

  const addLocationToCustomList = React.useCallback(
    async (listId: string, location: RelayLocationGeographical) => {
      const customList = getCustomListById(listId);
      if (customList === undefined) {
        return;
      }
      const updatedList = {
        ...customList,
        locations: [...customList.locations, location],
      };

      await updateCustomList(updatedList);
    },
    [getCustomListById, updateCustomList],
  );

  return addLocationToCustomList;
}
