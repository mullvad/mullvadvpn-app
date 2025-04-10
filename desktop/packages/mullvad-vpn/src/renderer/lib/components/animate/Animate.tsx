import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import { TransientProps } from '../../types';
import { useMounted } from '../../utility-hooks';
import { AnimateProvider } from './AnimateContext';
import { useAnimations, useHandleAnimationEnd, useShow } from './hooks';
import { Animation } from './types';

type AnimateBaseProps = {
  initial?: boolean;
  present?: boolean;
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
  TransientProps<Omit<AnimateBaseProps, 'animations'>> & {
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
export function Animate({ initial, present, children, ...props }: AnimateProps) {
  return (
    <AnimateProvider initial={initial} present={present}>
      <AnimateImpl {...props} present={present} initial={initial}>
        {children}
      </AnimateImpl>
    </AnimateProvider>
  );
}

function AnimateImpl({
  initial = true,
  present: presentProp,
  animations: animationsProp,
  duration,
  timingFunction,
  direction,
  iterationCount,
  onAnimationEnd,
  ...props
}: AnimateProps) {
  const animations = useAnimations(animationsProp);
  const show = useShow();
  const [initialPresent] = React.useState(presentProp);
  const [presentChanged, setPresentChanged] = React.useState(false);
  const [present, setPresent] = React.useState(presentProp);

  React.useEffect(() => {
    if (presentProp !== initialPresent && !presentChanged) {
      setPresentChanged(true);
    }
  }, [initialPresent, presentProp, presentChanged]);

  React.useEffect(() => {
    setPresent(presentProp);
  }, [presentProp]);

  const handleAnimationEnd = useHandleAnimationEnd(onAnimationEnd);
  const mounted = useMounted();
  if (!show) return null;

  return (
    <StyledDiv
      data-animate={initial || (mounted() && presentChanged)}
      data-present={present}
      data-is-present-animation={presentProp !== undefined}
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
