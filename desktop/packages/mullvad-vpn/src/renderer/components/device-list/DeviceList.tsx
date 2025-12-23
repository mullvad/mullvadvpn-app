import styled, { css } from 'styled-components';

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

const StyledDeviceListItem = styled(DeviceListItem)<{ $firstItem: boolean; $lastItem: boolean }>`
  ${({ $firstItem, $lastItem }) => css`
    ${StyledListItemItem} {
      --border-radius: ${Radius.radius12};
      transition: border-radius 0.15s ease-out;
      border-radius: 0;

      ${() => {
        if ($firstItem) {
          return css`
            border-top-left-radius: var(--border-radius);
            border-top-right-radius: var(--border-radius);
          `;
        }
        return null;
      }}

      ${() => {
        if ($lastItem) {
          return css`
            border-bottom-left-radius: var(--border-radius);
            border-bottom-right-radius: var(--border-radius);
          `;
        }
        return null;
      }}
    }
  `}
`;

export function DeviceList({ devices }: DeviceListProps) {
  const currentDeviceName = useSelector((state) => state.account.deviceName);
  const currentDevice = devices.find((device) => device.name === currentDeviceName);
  const nonCurrentDevices = devices.filter((device) => device.name !== currentDeviceName);

  return (
    <>
      {currentDevice && <DeviceListItem device={currentDevice} />}
      <StyledAnimatedList>
        {nonCurrentDevices.map((device, index) => {
          return (
            <AnimatedList.Item key={device.id}>
              <StyledDeviceListItem
                $firstItem={index === 0}
                $lastItem={index === nonCurrentDevices.length - 1}
                device={device}
              />
            </AnimatedList.Item>
          );
        })}
      </StyledAnimatedList>
    </>
  );
}
