import styled from 'styled-components';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { AnimatedList } from '../../lib/components/animated-list';
import { ListItemPositions } from '../../lib/components/list-item';
import { StyledListItemItem } from '../../lib/components/list-item/components';
import { useSelector } from '../../redux/store';
import { DeviceListItem } from '../device-list-item';

export type DeviceListProps = {
  devices: IDevice[];
};

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
`;

const StyledAnimatedListItem = styled(AnimatedList.Item)`
  ${StyledListItemItem} {
    transition: border-radius 0.15s ease-out;
  }
`;

export function DeviceList({ devices }: DeviceListProps) {
  const currentDeviceName = useSelector((state) => state.account.deviceName);
  const currentDevice = devices.find((device) => device.name === currentDeviceName);
  const nonCurrentDevices = devices.filter((device) => device.name !== currentDeviceName);

  return (
    <>
      {currentDevice && <DeviceListItem device={currentDevice} />}
      <StyledAnimatedList>
        {nonCurrentDevices.map((device, idx) => {
          let position: ListItemPositions | undefined = 'middle';
          if (idx === 0) {
            position = 'first';
          } else if (idx === nonCurrentDevices.length - 1) {
            position = 'last';
          }
          return (
            <StyledAnimatedListItem key={device.id}>
              <DeviceListItem position={position} device={device} />
            </StyledAnimatedListItem>
          );
        })}
      </StyledAnimatedList>
    </>
  );
}
