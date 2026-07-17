import styled from 'styled-components';

import { Flex } from '../../../../../../../lib/components';
import { useSpaceAllocationShifter } from '../../SpaceAllocationShifterContext';

const StyledSpaceAllocationShifterTarget = styled(Flex)``;

export function SpaceAllocationShifterTarget() {
  const { sourceHeight } = useSpaceAllocationShifter();

  return (
    <StyledSpaceAllocationShifterTarget
      style={{
        height: `${sourceHeight}px`,
      }}
    />
  );
}
