import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import { TransientProps } from '../../types';
import { AnimateProvider, useAnimateContext } from './AnimateContext';
import { useAnimate, useAnimations, useHandleAnimationEnd } from './hooks';
import { Animation } from './types';

type AnimateBaseProps = {
  initial?: boolean;
  present: boolean;
  duration?: React.CSSProperties['animationDuration'];
  timingFunction?: React.CSSProperties['animationTimingFunction'];
  direction?: React.CSSProperties['animationDirection'];
  iterationCount?: React.CSSProperties['animationIterationCount'];

  animations: Animation[];
  children?: React.ReactNode;
};

export type AnimateProps = AnimateBaseProps &
  Omit<React.HTMLAttributes<HTMLDivElement>, keyof AnimateBaseProps>;

const StyledDiv = styled.div<
  TransientProps<Omit<AnimateBaseProps, 'animations' | 'present'>> & {
    $animations: RuleSet;
  }
>`
  ${({
    $animations,
    $duration = '0.25s',
    $timingFunction = 'ease',
    $direction = 'normal',
    $iterationCount = '1',
  }) => {
    return css`
      // If the user prefers reduced motion, visibility still needs
      // to be toggled, otherwise this is handled by animations
      &&[data-is-present-animation='true'] {
        display: none;

        &&[data-present='true'] {
          display: block;
        }
      }
      @media (prefers-reduced-motion: no-preference) {
        &&[data-animate='true'] {
          --duration: ${$duration};
          --timing-function: ${$timingFunction};
          --direction: ${$direction};
          --iteration-count: ${$iterationCount};

          interpolate-size: allow-keywords;
          transition-behavior: allow-discrete;

          overflow: clip;
          ${$animations}
        }
      }
    `;
  }}
`;

/**
 * Animate that applies animation to a wrapper around it's children.
 *
 * @param initial - Whether animation should trigger on mount.
 * @param present - Whether element is present, i.e rendered or not.
 * @param animations - List of animations to apply.
 */
export function Animate({ animations, initial, present, children, ...props }: AnimateProps) {
  return (
    <AnimateProvider animations={animations} initial={initial} present={present}>
      <AnimateImpl {...props}>{children}</AnimateImpl>
    </AnimateProvider>
  );
}

export type AnimateImplProps = Omit<AnimateProps, 'animations' | 'present'>;

function AnimateImpl({
  duration,
  timingFunction,
  direction,
  iterationCount,
  onAnimationEnd,
  ...props
}: AnimateImplProps) {
  const { animatePresent } = useAnimateContext();
  const animations = useAnimations();
  const animate = useAnimate();
  const handleAnimationEnd = useHandleAnimationEnd();

  return (
    <StyledDiv
      data-animate={animate}
      data-present={animatePresent}
      data-is-present-animation={true}
      onAnimationEnd={handleAnimationEnd}
      $animations={animations}
      $duration={duration}
      $timingFunction={timingFunction}
      $direction={direction}
      $iterationCount={iterationCount}
      {...props}
    />
  );
}
