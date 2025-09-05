import { useCallback } from 'react';

import { useSplitTunnelingContext } from '../../../SplitTunnelingContext';

export function useScrollToTop() {
  const { scrollbarsRef } = useSplitTunnelingContext();

  const scrollToTop = useCallback(() => scrollbarsRef.current?.scrollToTop(true), [scrollbarsRef]);

  return scrollToTop;
}
