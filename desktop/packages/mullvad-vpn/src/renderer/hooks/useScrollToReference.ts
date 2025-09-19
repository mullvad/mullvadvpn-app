import { useEffect } from 'react';

export const useScrollToReference = <T extends Element = HTMLDivElement>(
  ref?: React.RefObject<T | null>,
  scroll?: boolean,
  onScrolled?: () => void,
) => {
  useEffect(() => {
    if (scroll) {
      ref?.current?.scrollIntoView({ behavior: 'instant', block: 'start' });
      onScrolled?.();
    }
  }, [onScrolled, ref, scroll]);
};
