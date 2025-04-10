import React from 'react';

import { AnimateProps } from '../Animate';
import { useAnimateContext } from '../AnimateContext';

export const useHandleAnimationEnd = (onAnimationEnd: AnimateProps['onAnimationEnd']) => {
  const { present, show, setShow } = useAnimateContext();
  return React.useCallback(
    (e: React.AnimationEvent<HTMLDivElement>) => {
      if (!present && show) {
        setShow(false);
      }
      if (onAnimationEnd) {
        onAnimationEnd(e);
      }
    },
    [onAnimationEnd, present, setShow, show],
  );
};
