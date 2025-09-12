import React from 'react';

import { useAppContext } from '../../../../context';
import { useSelector } from '../../../../redux/store';

export const useGetDevices = () => {
  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSelector((state) => state.account.devices);

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
