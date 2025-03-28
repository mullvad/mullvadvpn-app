import React from 'react';
import styled, { css } from 'styled-components';

import { TransientProps } from '../../types';
import { useAnimations } from './hooks';

export type Animation = FadeAnimation | WipeAnimation;

export type FadeAnimation = {
  type: 'fade';
};

export type WipeAnimation = {
  type: 'wipe';
  direction: 'vertical';
};

export type AnimateProps = React.HTMLAttributes<HTMLDivElement> & {
  present?: boolean;
  duration?: React.CSSProperties['animationDuration'];
  timingFunction?: React.CSSProperties['animationTimingFunction'];
  direction?: React.CSSProperties['animationDirection'];
  iterationCount?: React.CSSProperties['animationIterationCount'];

  animations: Animation[];
  children?: React.ReactNode;
};

const StyledDiv = styled.div<TransientProps<AnimateProps>>`
  ${({
    $animations,
    $duration = '0.25s',
    $timingFunction = 'ease',
    $direction = 'normal',
    $iterationCount = '1',
  }) => {
    const animations = useAnimations($animations);
    return css`
      &&[data-present-animation='true'] {
        display: none;

        &&[data-show='true'] {
          display: block;
        }
      }
      @media (prefers-reduced-motion: no-preference) {
        --duration: ${$duration};
        --timing-function: ${$timingFunction};
        --direction: ${$direction};
        --iteration-count: ${$iterationCount};

        interpolate-size: allow-keywords;
        transition-behavior: allow-discrete;

        overflow: clip;
        ${animations}
      }
    `;
  }}
`;

export function Animate({
  present,
  animations,
  duration,
  timingFunction,
  direction,
  iterationCount,
  ...props
}: AnimateProps) {
  return (
    <StyledDiv
      data-show={present}
      data-present-animation={present === undefined ? false : true}
      $animations={animations}
      $duration={duration}
      $timingFunction={timingFunction}
      $direction={direction}
      $iterationCount={iterationCount}
      {...props}
    />
  );
}
