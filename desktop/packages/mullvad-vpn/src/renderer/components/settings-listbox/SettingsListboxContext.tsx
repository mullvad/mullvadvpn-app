import React from 'react';

import { ScrollToAnchorId } from '../../../shared/ipc-types';

type SettingsListboxContextProps = Omit<SettingsListboxProviderProps, 'children'>;

const SettingsListboxContext = React.createContext<SettingsListboxContextProps | undefined>(
  undefined,
);

export const useSettingsListboxContext = (): SettingsListboxContextProps => {
  const context = React.useContext(SettingsListboxContext);
  if (!context) {
    throw new Error('useSettingsListboxContext must be used within a SettingsListboxProvider');
  }
  return context;
};

type SettingsListboxProviderProps = {
  anchorId?: ScrollToAnchorId;
  children: React.ReactNode;
};

export function SettingsListboxProvider({ children, ...props }: SettingsListboxProviderProps) {
  return (
    <SettingsListboxContext.Provider value={props}>{children}</SettingsListboxContext.Provider>
  );
}
