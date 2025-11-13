import React from 'react';
import styled from 'styled-components';

import { colors, Radius } from '../../../foundations';
import { Flex, FlexProps } from '../../flex';
import { useProgress } from '../ProgressContext';

const StyledFlex = styled(Flex)`
  background-color: ${colors.blue10};
  border-radius: ${Radius.radius4};
  width: 100%;
  height: 8px;
  overflow: hidden;
  position: relative;
`;

export type ProgressTrackProps = FlexProps;

export const ProgressTrack: React.FC<ProgressTrackProps> = ({ children, ...props }) => {
  const { max, min, value } = useProgress();
  return (
    <StyledFlex
      alignItems="center"
      role="progressbar"
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      {...props}>
      {children}
    </StyledFlex>
  );
};
