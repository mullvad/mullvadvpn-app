import React from 'react';

import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';

export function useDeleteCustomList() {
  const { deleteCustomList } = useAppContext();

  return React.useCallback(
    async (id: string) => {
      try {
        await deleteCustomList(id);
        return {
          success: true,
        };
      } catch (e) {
        const error = e as Error;
        log.error(`Failed to delete custom list ${id}: ${error.message}`);
        return {
          success: false,
        };
      }
    },
    [deleteCustomList],
  );
}
