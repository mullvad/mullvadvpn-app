import React from 'react';

import type { ICustomList } from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';

export function useUpdateCustomList() {
  const { updateCustomList } = useAppContext();

  return React.useCallback(
    async (updatedCustomList: ICustomList) => {
      try {
        const result = await updateCustomList(updatedCustomList);
        if (result) {
          return {
            success: false,
            error: result,
          };
        }
        return {
          success: true,
        };
      } catch (e) {
        const error = e as Error;
        log.error('Failed to update list:', error.message);
        return {
          success: false,
        };
      }
    },

    [updateCustomList],
  );
}
