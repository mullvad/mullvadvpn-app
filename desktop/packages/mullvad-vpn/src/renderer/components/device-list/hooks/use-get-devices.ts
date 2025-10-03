import React from 'react';

import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';
import { useSortedDevices } from './use-sorted-devices';

export const useGetDevices = () => {
  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSortedDevices();

  React.useEffect(() => {
    try {
      void fetchDevices(accountNumber);
    } catch {
      /* no-op */
    }
  }, [accountNumber, fetchDevices]);

  return devices;
};
