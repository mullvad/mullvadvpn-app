import React from 'react';
import styled from 'styled-components';

import { useMeasure } from '../../../../../../../hooks';
import { useSpaceAllocationShifter } from '../../SpaceAllocationShifterContext';

const StyledSpaceAllocationShifterSourceContainer = styled.div`
  position: relative;
`;

const StyledSpaceAllocationShifterSource = styled.div`
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  z-index: 1;
`;

export type SpaceAllocationShifterSourceProps = React.PropsWithChildren<{
  onMeasure: (number: number) => number;
}>;

export function SpaceAllocationShifterSource({
  children,
  onMeasure,
}: SpaceAllocationShifterSourceProps) {
  const { setSourceHeight } = useSpaceAllocationShifter();
  const [measureRef, { height }] = useMeasure();

  React.useLayoutEffect(() => {
    setSourceHeight(onMeasure(height));
  }, [height, onMeasure, setSourceHeight]);

  return (
    <StyledSpaceAllocationShifterSourceContainer>
      <StyledSpaceAllocationShifterSource ref={measureRef}>
        {children}
      </StyledSpaceAllocationShifterSource>
    </StyledSpaceAllocationShifterSourceContainer>
  );
}
