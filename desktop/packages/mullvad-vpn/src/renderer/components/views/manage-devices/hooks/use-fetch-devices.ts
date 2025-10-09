import React from 'react';

import { useAppContext } from '../../../../context';
import { useSelector } from '../../../../redux/store';

export const useFetchDevices = () => {
  const { fetchDevices: contextFetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;

  const getDevices = React.useCallback(() => {
    return contextFetchDevices(accountNumber);
  }, [accountNumber, contextFetchDevices]);

  return getDevices;
};
