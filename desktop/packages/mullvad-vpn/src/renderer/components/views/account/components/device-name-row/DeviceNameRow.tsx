import styled from 'styled-components';

import { Flex, Text } from '../../../../../lib/components';
import { useSelector } from '../../../../../redux/store';
import DeviceInfoButton from '../../../../DeviceInfoButton';

const StyledText = styled(Text)`
  text-transform: capitalize;
`;

export function DeviceNameRow() {
  const deviceName = useSelector((state) => state.account.deviceName);

  return (
    <Flex $justifyContent="space-between">
      <Flex $gap="small" $alignItems="center">
        <StyledText variant="bodySmallSemibold">{deviceName}</StyledText>
        <DeviceInfoButton />
      </Flex>
    </Flex>
  );
}
