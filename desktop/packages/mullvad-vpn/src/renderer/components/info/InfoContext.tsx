import React from 'react';

import type { InfoProps } from './Info';

type InfoContextProps = Required<Omit<InfoProviderProps, 'children'>>;

const InfoContext = React.createContext<InfoContextProps | undefined>(undefined);

export const useInfoContext = (): InfoContextProps => {
  const context = React.useContext(InfoContext);
  if (!context) {
    throw new Error('useInfoContext must be used within a InfoProvider');
  }
  return context;
};

type InfoProviderProps = React.PropsWithChildren & Pick<InfoProps, 'open' | 'onOpenChange'>;

export function InfoProvider({ open, onOpenChange, children, ...props }: InfoProviderProps) {
  const [openState, onOpenChangeState] = React.useState(false);

  const value = React.useMemo(
    () => ({
      open: open ?? openState,
      onOpenChange: onOpenChange ?? onOpenChangeState,
      ...props,
    }),
    [open, openState, onOpenChange, props],
  );

  return <InfoContext.Provider value={value}>{children}</InfoContext.Provider>;
}
