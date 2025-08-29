import { useCallback, useRef } from 'react';

import { useAnimateContext } from '../AnimateContext';

export const useHandleAnimationEnd = () => {
  const { animations, present, setAnimate, setAnimatePresent } = useAnimateContext();
  const animationsCount = animations.length;
  const animationsFinishedCount = useRef(0);

  const handleAnimationEnd = useCallback(() => {
    const nextAnimationsFinishedCount = animationsFinishedCount.current + 1;

    if (nextAnimationsFinishedCount === animationsCount) {
      animationsFinishedCount.current = 0;
      setAnimate(false);
      setAnimatePresent(present);
    } else {
      animationsFinishedCount.current = nextAnimationsFinishedCount;
    }
  }, [animationsCount, animationsFinishedCount, present, setAnimate, setAnimatePresent]);

  return handleAnimationEnd;
};
