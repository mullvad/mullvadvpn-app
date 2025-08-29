import { useEffect } from 'react';

import { useAnimateContext } from '../AnimateContext';
import { usePreviousValue } from './usePreviousValue';

export const useAnimate = () => {
  const { animate, present, setAnimate, setAnimatePresent } = useAnimateContext();
  const previousPresent = usePreviousValue(present);

  useEffect(() => {
    if (present !== previousPresent) {
      setAnimate(true);
      setAnimatePresent(present);
    }
  }, [present, previousPresent, setAnimate, setAnimatePresent]);

  return animate;
};
