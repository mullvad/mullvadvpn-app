import React from 'react';

import { CheckboxProps } from './Checkbox';

type CheckboxContextProps = Omit<CheckboxProviderProps, 'children'>;

const CheckboxContext = React.createContext<CheckboxContextProps | undefined>(undefined);

export const useCheckboxContext = (): CheckboxContextProps => {
  const context = React.useContext(CheckboxContext);
  if (!context) {
    throw new Error('useCheckboxContext must be used within a CheckboxProvider');
  }
  return context;
};

type CheckboxProviderProps = React.PropsWithChildren<
  Pick<CheckboxProps, 'disabled' | 'checked' | 'onCheckedChange' | 'inputId' | 'descriptionId'>
>;

export function CheckboxProvider({
  inputId: inputIdProp,
  descriptionId: descriptionIdProp,
  children,
  ...props
}: CheckboxProviderProps) {
  const inputId = React.useId();
  const descriptionId = React.useId();
  return (
    <CheckboxContext.Provider
      value={{
        ...props,
        inputId: inputIdProp ?? inputId,
        descriptionId: descriptionIdProp ?? descriptionId,
      }}>
      {children}
    </CheckboxContext.Provider>
  );
}
