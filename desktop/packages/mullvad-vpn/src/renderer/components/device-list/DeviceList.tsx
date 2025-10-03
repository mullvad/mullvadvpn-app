import { IDevice } from '../../../shared/daemon-rpc-types';
import { Flex, Spinner } from '../../lib/components';
import { AnimatedList } from '../../lib/components/animated-list';
import { DeviceListItem } from '../device-list-item';
import { useGetDevices } from './hooks';

const getDeviceKey = (device: IDevice): string => device.id;

export function DeviceList() {
  const devices = useGetDevices();
  const showList = devices.length > 0;
  return (
    <Flex $flexDirection="column" $alignItems="center">
      {!showList && <Spinner size="big" />}
      {showList && (
        <AnimatedList>
          {devices.map((device) => (
            <AnimatedList.Item key={getDeviceKey(device)}>
              <DeviceListItem device={device} />
            </AnimatedList.Item>
          ))}
        </AnimatedList>
      )}
    </Flex>
  );
}
