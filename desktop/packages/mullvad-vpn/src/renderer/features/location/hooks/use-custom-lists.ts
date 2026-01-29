import React from 'react';

import type { CustomListError } from '../../../../shared/daemon-rpc-types';
import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';

export function useCustomLists() {
  const customLists = useSelector((state) => state.settings.customLists);
  const {
    createCustomList: contextCreateCustomList,
    updateCustomList,
    deleteCustomList,
  } = useAppContext();

  const createCustomList = React.useCallback(
    async (name: string): Promise<void | CustomListError> => {
      return contextCreateCustomList({
        name,
        locations: [],
      });
    },
    [contextCreateCustomList],
  );

  return { customLists, createCustomList, updateCustomList, deleteCustomList };
}
