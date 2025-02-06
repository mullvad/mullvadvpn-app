import React from 'react';
import styled from 'styled-components';

import { Radius, Spacings } from '../../foundations';
import { Flex } from '../flex';
import { ProgressContentGroup, ProgressPercent, ProgressRange, ProgressText } from './components';
import { ProgressProvider } from './ProgressContext';

export interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  min?: number;
  max?: number;
  value: number;
  disabled?: boolean;
}

const ProgressTrack = styled(Flex)`
  // TODO: Replace with token when available
  background-color: ${'rgba(48, 67, 88, 1)'};
  border-radius: ${Radius.radius4};
  width: 100%;
  height: 24px;
  overflow: hidden;
  position: relative;
  && ${ProgressContentGroup} {
    position: absolute;
    right: ${Spacings.spacing3};
  }
`;

const Progress = React.forwardRef<HTMLDivElement, ProgressProps>(
  ({ min = 0, max = 100, value, disabled, ...props }, ref) => {
    const normalizedValue = Math.min(Math.max(value, min), max);
    const percent = ((normalizedValue - min) / (max - min)) * 100;
    return (
      <ProgressProvider value={value} min={min} max={max} percent={percent} disabled={disabled}>
        <ProgressTrack
          $alignItems="center"
          role="progressbar"
          aria-valuemin={min}
          aria-valuemax={max}
          aria-valuenow={value}
          ref={ref}
          {...props}
        />
      </ProgressProvider>
    );
  },
);

Progress.displayName = 'Progress';

const ProgressNamespace = Object.assign(Progress, {
  ContentGroup: ProgressContentGroup,
  Percent: ProgressPercent,
  Range: ProgressRange,
  Text: ProgressText,
});

export { ProgressNamespace as Progress };
