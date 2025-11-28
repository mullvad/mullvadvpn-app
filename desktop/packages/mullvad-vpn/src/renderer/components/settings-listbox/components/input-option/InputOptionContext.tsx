import React, { createContext, ReactNode, useContext } from 'react';

import { UseTextFieldState } from '../../../../lib/components/text-field';

type InputOptionContextType = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  triggerRef: React.RefObject<HTMLLIElement | null>;
  labelId: string | undefined;
  inputState: UseTextFieldState;
};

const InputOptionContextContext = createContext<InputOptionContextType | undefined>(undefined);

type InputOptionProviderProps = {
  children: ReactNode;
} & InputOptionContextType;

export const InputOptionProvider = ({ children, ...props }: InputOptionProviderProps) => {
  return (
    <InputOptionContextContext.Provider value={props}>
      {children}
    </InputOptionContextContext.Provider>
  );
};

export const useInputOptionContext = (): InputOptionContextType => {
  const context = useContext(InputOptionContextContext);
  if (!context) {
    throw new Error('useInputOptionContext must be used within an InputOptionProvider');
  }
  return context;
};
