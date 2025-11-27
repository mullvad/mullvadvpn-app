import { useEffect } from 'react';

import { useEffectEvent } from '../utility-hooks';

export function useInterval(fn: () => void, interval: number) {
  const fnEvent = useEffectEvent(fn);

  useEffect(() => {
    const id = setInterval(fnEvent, interval);

    return () => {
      clearInterval(id);
    };

    // eslint-disable-next-line react-compiler/react-compiler
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [interval]);
}
