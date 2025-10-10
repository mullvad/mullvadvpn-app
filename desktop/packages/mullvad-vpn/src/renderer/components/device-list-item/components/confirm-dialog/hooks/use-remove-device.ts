import React from 'react';

import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';

export const useRemoveDevice = () => {
  const { removeDevice: contextRemoveDevice } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const removeDevice = React.useCallback(
    async (deviceId: string) => {
      await contextRemoveDevice({ accountNumber, deviceId });
    },
    [contextRemoveDevice, accountNumber],
  );

  return removeDevice;
};
