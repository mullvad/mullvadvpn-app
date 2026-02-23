import styled from 'styled-components';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { AnimatedList } from '../../lib/components/animated-list';
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

const StyledAnimatedListItemRoot = styled(AnimatedList.Item)``;
const StyledAnimatedListItem = styled(StyledAnimatedListItemRoot)`
  ${StyledListItemItem} {
    transition: border-radius 0.15s ease-out;
  }
  // If preceded by another item, remove the top border radius
  ${StyledAnimatedListItemRoot} + & {
    margin-top: 1px;
    ${StyledListItemItem} {
      border-top-left-radius: 0;
      border-top-right-radius: 0;
    }
  }

  // If followed by another item, remove the bottom border radius
  &:has(~ ${StyledAnimatedListItemRoot}) {
    ${StyledListItemItem} {
      border-bottom-left-radius: 0;
      border-bottom-right-radius: 0;
    }
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
        {nonCurrentDevices.map((device) => {
          return (
            <StyledAnimatedListItem key={device.id}>
              <DeviceListItem device={device} />
            </StyledAnimatedListItem>
          );
        })}
      </StyledAnimatedList>
    </>
  );
}
