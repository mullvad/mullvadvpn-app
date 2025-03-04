import React from 'react';

import { Spacings } from '../../foundations';
import { Flex } from '../flex';
import {
  ProgressFooter,
  ProgressPercent,
  ProgressRange,
  ProgressText,
  ProgressTrack,
} from './components';
import { ProgressProvider } from './ProgressContext';

export interface ProgressProps extends React.HTMLAttributes<HTMLDivElement> {
  min?: number;
  max?: number;
  value: number;
  disabled?: boolean;
}

const Progress = React.forwardRef<HTMLDivElement, ProgressProps>(
  ({ min = 0, max = 100, value, disabled, children, ...props }, ref) => {
    const normalizedValue = Math.min(Math.max(value, min), max);
    const percent = ((normalizedValue - min) / (max - min)) * 100;
    return (
      <ProgressProvider value={value} min={min} max={max} percent={percent} disabled={disabled}>
        <Flex $flexDirection="column" $gap={Spacings.small} ref={ref} {...props}>
          {children}
        </Flex>
      </ProgressProvider>
    );
  },
);

Progress.displayName = 'Progress';

const ProgressNamespace = Object.assign(Progress, {
  Footer: ProgressFooter,
  Track: ProgressTrack,
  Percent: ProgressPercent,
  Range: ProgressRange,
  Text: ProgressText,
});

export { ProgressNamespace as Progress };
