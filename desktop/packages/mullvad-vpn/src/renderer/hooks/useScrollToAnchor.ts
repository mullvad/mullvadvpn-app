import { useEffect } from 'react';

export const useScrollToReference = (
  ref?: React.RefObject<HTMLDivElement | null>,
  scroll?: boolean,
) => {
  useEffect(() => {
    if (scroll) {
      ref?.current?.scrollIntoView({ behavior: 'instant', block: 'start' });
    }
  }, [ref, scroll]);
};
