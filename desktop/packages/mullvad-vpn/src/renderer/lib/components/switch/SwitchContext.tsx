import React from 'react';

import { SwitchProps } from './Switch';

type SwitchContextProps = Omit<SwitchProviderProps, 'children'>;

const SwitchContext = React.createContext<SwitchContextProps | undefined>(undefined);

export const useSwitchContext = (): SwitchContextProps => {
  const context = React.useContext(SwitchContext);
  if (!context) {
    throw new Error('useSwitchContext must be used within a SwitchProvider');
  }
  return context;
};

type SwitchProviderProps = React.PropsWithChildren<
  Pick<SwitchProps, 'disabled' | 'checked' | 'onCheckedChange' | 'inputId' | 'descriptionId'>
>;

export function SwitchProvider({
  inputId: inputIdProp,
  descriptionId: descriptionIdProp,
  children,
  ...props
}: SwitchProviderProps) {
  const inputId = React.useId();
  const descriptionId = React.useId();
  return (
    <SwitchContext.Provider
      value={{
        ...props,
        inputId: inputIdProp ?? inputId,
        descriptionId: descriptionIdProp ?? descriptionId,
      }}>
      {children}
    </SwitchContext.Provider>
  );
}
