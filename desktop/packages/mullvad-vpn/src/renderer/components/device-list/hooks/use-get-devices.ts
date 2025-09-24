import React from 'react';

import { useAppContext } from '../../../context';
import { useSelector } from '../../../redux/store';
import { useSortedDevices } from './use-sorted-devices';

export const useGetDevices = () => {
  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSortedDevices();

  const [loading, setLoading] = React.useState(false);

  React.useEffect(() => {
    setLoading(true);
    try {
      void fetchDevices(accountNumber);
    } catch {
      /* no-op */
    } finally {
      setLoading(false);
    }
  }, [accountNumber, fetchDevices]);

  return { loading, devices };
};
