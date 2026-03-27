import React from 'react';

import type { RelayLocationGeographical } from '../../../../shared/daemon-rpc-types';
import log from '../../../../shared/logging';
import { useAppContext } from '../../../context';

export function useCreateCustomList() {
  const { createCustomList: contextCreateCustomList } = useAppContext();

  const createCustomList = React.useCallback(
    async (name: string, locations: RelayLocationGeographical[] = []) => {
      try {
        const result = await contextCreateCustomList({
          name,
          locations,
        });
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
        log.error('Failed to create list:', error.message);
        return {
          success: false,
        };
      }
    },
    [contextCreateCustomList],
  );

  return createCustomList;
}
