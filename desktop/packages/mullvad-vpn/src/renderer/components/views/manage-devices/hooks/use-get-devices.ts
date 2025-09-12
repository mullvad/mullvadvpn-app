import React from 'react';

import { useAppContext } from '../../../../context';
import { useSelector } from '../../../../redux/store';

export const useGetDevices = () => {
  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSelector((state) => state.account.devices);

  React.useEffect(() => {
    try {
      void fetchDevices(accountNumber);
    } catch {
      /* no-op */
    }
  }, [accountNumber, fetchDevices]);

  return devices;
};
