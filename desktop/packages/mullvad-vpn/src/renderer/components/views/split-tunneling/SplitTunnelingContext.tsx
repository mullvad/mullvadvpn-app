import React, { useMemo, useState } from 'react';

import { useStyledRef } from '../../../lib/utility-hooks';
import { type CustomScrollbarsRef } from '../../CustomScrollbars';

type SplitTunnelingContextProviderProps = {
  children: React.ReactNode;
};

type SplitTunnelingContext = {
  browsing: boolean;
  scrollbarsRef: React.RefObject<CustomScrollbarsRef | null>;
  setBrowsing: (value: boolean) => void;
};

const SplitTunnelingContext = React.createContext<SplitTunnelingContext | undefined>(undefined);

export const useSplitTunnelingContext = (): SplitTunnelingContext => {
  const context = React.useContext(SplitTunnelingContext);
  if (!context) {
    throw new Error('useSplitTunnelingContext must be used within a SplitTunnelingContext');
  }
  return context;
};

export function SplitTunnelingContextProvider({ children }: SplitTunnelingContextProviderProps) {
  const [browsing, setBrowsing] = useState(false);
  const scrollbarsRef = useStyledRef<CustomScrollbarsRef>();

  const value = useMemo(
    () => ({
      browsing,
      scrollbarsRef,
      setBrowsing,
    }),
    [browsing, scrollbarsRef],
  );

  return <SplitTunnelingContext value={value}>{children}</SplitTunnelingContext>;
}
