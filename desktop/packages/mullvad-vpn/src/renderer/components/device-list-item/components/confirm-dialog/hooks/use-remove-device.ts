import React from 'react';

import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';

export const useRemoveDevice = () => {
  const { removeDevice } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const onRemoveDevice = React.useCallback(
    async (deviceId: string) => {
      await removeDevice({ accountNumber, deviceId });
    },
    [removeDevice, accountNumber],
  );

  return onRemoveDevice;
};
