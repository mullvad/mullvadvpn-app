import { IDevice } from '../../../shared/daemon-rpc-types';
import { AnimatedList } from '../../lib/components/animated-list';
import { DeviceListItem } from '../device-list-item';

export type DeviceListProps = {
  devices: IDevice[];
};

export function DeviceList({ devices }: DeviceListProps) {
  return (
    <AnimatedList>
      {devices.map((device) => (
        <AnimatedList.Item key={device.id}>
          <DeviceListItem device={device} />
        </AnimatedList.Item>
      ))}
    </AnimatedList>
  );
}
