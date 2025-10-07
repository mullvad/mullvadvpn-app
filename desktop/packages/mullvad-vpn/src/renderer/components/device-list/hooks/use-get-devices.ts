import React from 'react';

import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';
import { useSortedDevices } from './use-sorted-devices';

export const useGetDevices = () => {
  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSortedDevices();

  React.useEffect(() => {
    void fetchDevices(accountNumber);
  }, [accountNumber, fetchDevices]);

  return devices;
};
