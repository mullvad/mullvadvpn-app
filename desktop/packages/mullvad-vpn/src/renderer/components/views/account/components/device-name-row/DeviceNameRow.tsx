import { Flex } from '../../../../../lib/components';
import { useSelector } from '../../../../../redux/store';
import DeviceInfoButton from '../../../../DeviceInfoButton';
import { DeviceRowValue } from '../../AccountStyles';

export function DeviceNameRow() {
  const deviceName = useSelector((state) => state.account.deviceName);
  return (
    <Flex $gap="small" $alignItems="center">
      <DeviceRowValue>{deviceName}</DeviceRowValue>
      <DeviceInfoButton />
    </Flex>
  );
}
