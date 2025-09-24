import React from 'react';

import { useSelector } from '../../../../redux/store';

export const useSortedDevices = () => {
  const devices = useSelector((state) => state.account.devices);
  const currentDeviceName = useSelector((state) => state.account.deviceName);
  return React.useMemo(() => {
    const currentDevice = devices.find((device) => device.name === currentDeviceName);
    if (!currentDevice) return devices;

    return [currentDevice, ...devices.filter((device) => device.name !== currentDeviceName)];
  }, [currentDeviceName, devices]);
};
