import { useEffect } from 'react';

import { useEffectEvent } from '../utility-hooks';

export function useInterval(fn: () => void, interval: number) {
  const fnEvent = useEffectEvent(fn);

  useEffect(() => {
    const id = setInterval(fnEvent, interval);

    return () => {
      clearInterval(id);
    };
  }, [interval]);
}
