import React, { createContext, useContext } from 'react';

import { TextFieldProps } from './TextField';

type TextFieldContextType = Omit<TextFieldProviderProps, 'children'> & {
  labelId: string;
};

const TextFieldContext = createContext<TextFieldContextType | undefined>(undefined);

type TextFieldProviderProps = React.PropsWithChildren<
  Pick<TextFieldProps, 'variant' | 'invalid' | 'disabled' | 'value' | 'onValueChange'>
>;

export const TextFieldProvider = ({ children, ...props }: TextFieldProviderProps) => {
  const labelId = React.useId();

  return (
    <TextFieldContext.Provider value={{ labelId, ...props }}>{children}</TextFieldContext.Provider>
  );
};

export const useTextFieldContext = (): TextFieldContextType => {
  const context = useContext(TextFieldContext);
  if (!context) {
    throw new Error('useTextField must be used within a TextFieldProvider');
  }
  return context;
};
