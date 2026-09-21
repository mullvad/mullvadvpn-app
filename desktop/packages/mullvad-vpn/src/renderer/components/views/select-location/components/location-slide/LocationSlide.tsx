import { type AnimationDefinition, type HTMLMotionProps, motion } from 'motion/react';
import React from 'react';
import styled from 'styled-components';

import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type LocationSlideProps = HTMLMotionProps<'div'>;

// Take up full space, needed for components such as `AutomaticLocation` to display correctly.
export const StyledLocationSlide = styled(motion.div)`
  display: flex;
  flex-direction: column;
  flex-grow: 1;
`;

export function LocationSlide({ children, ...props }: LocationSlideProps) {
  const { setTransitionState } = useSelectLocationViewContext();

  const handleAnimationStart = React.useCallback(
    (definition: AnimationDefinition) => {
      if (typeof definition === 'object' && 'opacity' in definition) {
        if (definition.opacity === 0) {
          setTransitionState('transitioningOut');
        }
        if (definition.opacity === 1) {
          setTransitionState('transitioningIn');
        }
      }
    },
    [setTransitionState],
  );

  const handleAnimationComplete = React.useCallback(
    (definition: AnimationDefinition) => {
      if (typeof definition === 'object' && 'opacity' in definition) {
        if (definition.opacity === 1) {
          setTransitionState('idle');
        }
      }
    },
    [setTransitionState],
  );

  return (
    <StyledLocationSlide
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.25 }}
      onAnimationStart={handleAnimationStart}
      onAnimationComplete={handleAnimationComplete}
      {...props}>
      {children}
    </StyledLocationSlide>
  );
}
