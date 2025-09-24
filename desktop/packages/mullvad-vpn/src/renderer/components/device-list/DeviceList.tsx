import { IDevice } from '../../../shared/daemon-rpc-types';
import { Flex, Spinner } from '../../lib/components';
import { DeviceListItem } from '../device-list-item';
import List from '../List';
import { useGetDevices } from './hooks';

const getDeviceKey = (device: IDevice): string => device.id;

export function DeviceList() {
  const devices = useGetDevices();
  const showList = devices.length > 0;
  return (
    <Flex $flexDirection="column" $alignItems="center">
      {showList ? (
        <List items={devices} getKey={getDeviceKey} skipInitialAddTransition>
          {(device) => <DeviceListItem device={device} />}
        </List>
      ) : (
        <Spinner size="big" />
      )}
    </Flex>
  );
}
