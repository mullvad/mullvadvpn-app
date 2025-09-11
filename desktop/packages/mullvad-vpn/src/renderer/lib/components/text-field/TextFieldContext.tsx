import { createContext, ReactNode, useContext } from 'react';

import { TextFieldProps } from './TextField';

type TextFieldContextType = TextFieldProps & {
  labelId: string;
};

const TextFieldContext = createContext<TextFieldContextType | undefined>(undefined);

type TextFieldProviderProps = TextFieldContextType & {
  children: ReactNode;
};

export const TextFieldProvider = ({ children, ...props }: TextFieldProviderProps) => {
  return <TextFieldContext.Provider value={props}>{children}</TextFieldContext.Provider>;
};

export const useTextFieldContext = (): TextFieldContextType => {
  const context = useContext(TextFieldContext);
  if (!context) {
    throw new Error('useTextField must be used within a TextFieldProvider');
  }
  return context;
};
