import { IDevice } from '../../../shared/daemon-rpc-types';
import { Spinner } from '../../lib/components';
import { DeviceListItem } from '../device-list-item';
import List from '../List';
import { useGetDevices } from './hooks';

const getDeviceKey = (device: IDevice): string => device.id;

export function DeviceList() {
  const { loading, devices } = useGetDevices();
  return (
    <div>
      {loading && <Spinner />}
      {!loading && (
        <List items={devices} getKey={getDeviceKey} skipAddTransition>
          {(device) => <DeviceListItem device={device} />}
        </List>
      )}
    </div>
  );
}
