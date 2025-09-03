import React, { useCallback, useMemo, useState } from 'react';

import { useStyledRef } from '../../../lib/utility-hooks';
import { type CustomScrollbarsRef } from '../../CustomScrollbars';

type SplitTunnelingContextProviderProps = {
  children: React.ReactNode;
};

type SplitTunnelingContext = {
  browsing: boolean;
  scrollbarsRef: React.RefObject<CustomScrollbarsRef | null>;
  scrollToTop: () => void;
  setBrowsing: (value: boolean) => void;
};

const SplitTunnelingContext = React.createContext<SplitTunnelingContext | undefined>(undefined);

export const useSplitTunnelingContext = (): SplitTunnelingContext => {
  const context = React.useContext(SplitTunnelingContext);
  if (!context) {
    throw new Error('useLinuxSettings must be used within a SplitTunnelingContext');
  }
  return context;
};

export function SplitTunnelingContextProvider({ children }: SplitTunnelingContextProviderProps) {
  const [browsing, setBrowsing] = useState(false);
  const scrollbarsRef = useStyledRef<CustomScrollbarsRef>();

  const scrollToTop = useCallback(() => scrollbarsRef.current?.scrollToTop(true), [scrollbarsRef]);

  const value = useMemo(
    () => ({
      browsing,
      scrollbarsRef,
      scrollToTop,
      setBrowsing,
    }),
    [browsing, scrollbarsRef, scrollToTop],
  );

  return <SplitTunnelingContext value={value}>{children}</SplitTunnelingContext>;
}
