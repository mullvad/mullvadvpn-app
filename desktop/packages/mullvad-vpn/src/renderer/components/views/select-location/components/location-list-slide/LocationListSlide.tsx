import { type HTMLMotionProps, motion } from 'motion/react';
import React from 'react';

import { useScrollPositionContext } from '../../ScrollPositionContext';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type LocationListSlideProps = HTMLMotionProps<'div'>;

export function LocationListSlide(props: LocationListSlideProps) {
  const { resetScroll } = useScrollPositionContext();
  const { setIsolatedItem } = useSelectLocationViewContext();

  const handleAnimationStart = React.useCallback(
    (definition: string) => {
      if (definition === 'enter') {
        resetScroll();
        setIsolatedItem(undefined);
      }
    },
    [resetScroll, setIsolatedItem],
  );

  return (
    <motion.div
      {...props}
      initial="exit"
      animate="enter"
      exit="exit"
      variants={{
        enter: { opacity: 1, x: 0 },
        exit: { opacity: 0, x: -50 },
      }}
      transition={{ duration: 0.2, delay: 0.05 }}
      onAnimationStart={handleAnimationStart}
    />
  );
}
