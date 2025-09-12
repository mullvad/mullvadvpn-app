import React from 'react';

import { useDeviceListItemContext } from '../../../DeviceListItemContext';
import { useRemoveDevice } from '../hooks';
import { useHandleRemoveDeviceError } from './use-handle-remove-device-error';

export const useHandleRemoveDevice = () => {
  const { device, setDeleting, hideConfirmDialog } = useDeviceListItemContext();
  const removeDevice = useRemoveDevice();
  const handleError = useHandleRemoveDeviceError();

  const handleRemoveDevice = React.useCallback(async () => {
    setDeleting();
    hideConfirmDialog();
    try {
      await removeDevice(device.id);
    } catch (e) {
      await handleError(e as Error);
    }
  }, [setDeleting, hideConfirmDialog, removeDevice, device.id, handleError]);
  return handleRemoveDevice;
};
