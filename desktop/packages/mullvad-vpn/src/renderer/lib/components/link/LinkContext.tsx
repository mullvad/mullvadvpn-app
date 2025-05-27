import React from 'react';

import { LinkProps } from './Link';

interface LinkContextProps {
  color?: LinkProps['color'];
  variant?: LinkProps['variant'];
}

const LinkContext = React.createContext<LinkContextProps | undefined>(undefined);

export const useLinkContext = (): LinkContextProps => {
  const context = React.useContext(LinkContext);
  if (!context) {
    throw new Error('useLinkContext must be used within a LinkProvider');
  }
  return context;
};

interface LinkProviderProps {
  color?: LinkContextProps['color'];
  variant?: LinkContextProps['variant'];
  children: React.ReactNode;
}

export function LinkProvider({ color, variant, children }: LinkProviderProps) {
  return <LinkContext.Provider value={{ color, variant }}>{children}</LinkContext.Provider>;
}
