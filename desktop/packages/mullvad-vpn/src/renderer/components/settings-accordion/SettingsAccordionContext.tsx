import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';

type SettingsAccordionContextProps = Omit<SettingsAccordionProviderProps, 'children'>;

const SettingsAccordionContext = React.createContext<SettingsAccordionContextProps | undefined>(
  undefined,
);

export const useSettingsAccordionContext = (): SettingsAccordionContextProps => {
  const context = React.useContext(SettingsAccordionContext);
  if (!context) {
    throw new Error('useSettingsAccordionContext must be used within a SettingsAccordionProvider');
  }
  return context;
};

type SettingsAccordionProviderProps = {
  anchorId?: ScrollToAnchorId;
  children: React.ReactNode;
};

export function SettingsAccordionProvider({ children, ...props }: SettingsAccordionProviderProps) {
  return (
    <SettingsAccordionContext.Provider value={props}>{children}</SettingsAccordionContext.Provider>
  );
}
