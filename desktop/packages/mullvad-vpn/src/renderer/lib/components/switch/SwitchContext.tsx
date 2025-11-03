import React from 'react';

import { SwitchProps } from './Switch';

interface SwitchContextProps {
  id: string;
  labelId: string;
  disabled: SwitchProps['disabled'];
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
}

const SwitchContext = React.createContext<SwitchContextProps | undefined>(undefined);

export const useSwitchContext = (): SwitchContextProps => {
  const context = React.useContext(SwitchContext);
  if (!context) {
    throw new Error('useSwitchContext must be used within a SwitchProvider');
  }
  return context;
};

interface SwitchProviderProps {
  id: string;
  labelId: string;
  disabled: SwitchProps['disabled'];
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
  children: React.ReactNode;
}

export function SwitchProvider({ children, ...props }: SwitchProviderProps) {
  return <SwitchContext.Provider value={props}>{children}</SwitchContext.Provider>;
}
