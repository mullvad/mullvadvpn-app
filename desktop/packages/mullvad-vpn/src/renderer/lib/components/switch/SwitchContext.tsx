import React from 'react';

import { SwitchProps } from './Switch';

type SwitchContextProps = SwitchProviderProps;

const SwitchContext = React.createContext<SwitchContextProps | undefined>(undefined);

export const useSwitchContext = (): SwitchContextProps => {
  const context = React.useContext(SwitchContext);
  if (!context) {
    throw new Error('useSwitchContext must be used within a SwitchProvider');
  }
  return context;
};

type SwitchProviderProps = React.PropsWithChildren<{
  disabled: SwitchProps['disabled'];
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
  labelId: SwitchProps['labelId'];
  descriptionId: SwitchProps['descriptionId'];
}>;

export function SwitchProvider({
  labelId: labelIdProp,
  descriptionId: descriptionIdProp,
  children,
  ...props
}: SwitchProviderProps) {
  const labelId = React.useId();
  const descriptionId = React.useId();
  return (
    <SwitchContext.Provider
      value={{
        ...props,
        labelId: labelIdProp ?? labelId,
        descriptionId: descriptionIdProp ?? descriptionId,
      }}>
      {children}
    </SwitchContext.Provider>
  );
}
