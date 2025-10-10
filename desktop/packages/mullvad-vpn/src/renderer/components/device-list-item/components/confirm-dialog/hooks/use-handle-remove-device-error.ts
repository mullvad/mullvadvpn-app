import React from 'react';

import { IDevice } from '../../../../../../shared/daemon-rpc-types';
import log from '../../../../../../shared/logging';
import { useAppContext } from '../../../../../context';
import { useSelector } from '../../../../../redux/store';
import { useDeviceListItemContext } from '../../../DeviceListItemContext';

export const useHandleRemoveDeviceError = () => {
  const { fetchDevices } = useAppContext();
  const {
    hideConfirmDialog,
    resetDeleting,
    setError,
    device: { id: deviceId },
  } = useDeviceListItemContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;

  const handleError = React.useCallback(
    async (error: Error) => {
      log.error(`Failed to remove device: ${error.message}`);

      let devices: Array<IDevice> | undefined = undefined;
      try {
        devices = await fetchDevices(accountNumber);
      } finally {
        if (devices === undefined || devices.some((device) => device.id === deviceId)) {
          hideConfirmDialog();
          resetDeleting();
          setError();
        }
      }
    },
    [fetchDevices, accountNumber, deviceId, hideConfirmDialog, resetDeleting, setError],
  );

  return handleError;
};
