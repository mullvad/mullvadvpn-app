import { useEffect } from 'react';

export function useInterval(fn: () => void, interval: number) {
  useEffect(() => {
    const id = setInterval(fn, interval);

    return () => {
      clearInterval(id);
    };
  });
}
