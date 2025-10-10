import { useSelector } from '../../../redux/store';
import { useDeviceListItemContext } from '../DeviceListItemContext';

export const useIsCurrentDevice = () => {
  const currentDevice = useSelector((state) => state.account.deviceName);
  const { device } = useDeviceListItemContext();
  return device.name === currentDevice;
};
