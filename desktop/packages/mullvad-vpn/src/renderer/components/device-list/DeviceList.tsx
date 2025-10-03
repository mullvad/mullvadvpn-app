import { Flex, Spinner } from '../../lib/components';
import { AnimatedList } from '../../lib/components/animated-list';
import { DeviceListItem } from '../device-list-item';
import { useGetDevices } from './hooks';

export function DeviceList() {
  const devices = useGetDevices();
  const showList = devices.length > 0;
  return (
    <Flex $flexDirection="column" $alignItems="center">
      {showList ? (
        <AnimatedList>
          {devices.map((device) => (
            <AnimatedList.Item key={device.id}>
              <DeviceListItem device={device} />
            </AnimatedList.Item>
          ))}
        </AnimatedList>
      ) : (
        <Spinner size="big" />
      )}
    </Flex>
  );
}
