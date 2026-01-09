import styled from 'styled-components';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { AnimatedList } from '../../lib/components/animated-list';
import { StyledListItemItem } from '../../lib/components/list-item/components';
import { Radius } from '../../lib/foundations';
import { useSelector } from '../../redux/store';
import { DeviceListItem } from '../device-list-item';

export type DeviceListProps = {
  devices: IDevice[];
};

const StyledAnimatedList = styled(AnimatedList)`
  display: flex;
  flex-direction: column;
  gap: 1px;
`;

const StyledAnimatedListItem = styled(AnimatedList.Item)`
  ${StyledListItemItem} {
    --border-radius: ${Radius.radius12};
    transition: border-radius 0.15s ease-out;
    border-radius: 0;
  }

  &&:first-child {
    ${StyledListItemItem} {
      border-top-left-radius: var(--border-radius);
      border-top-right-radius: var(--border-radius);
    }
  }

  &&:last-child {
    ${StyledListItemItem} {
      border-bottom-left-radius: var(--border-radius);
      border-bottom-right-radius: var(--border-radius);
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
