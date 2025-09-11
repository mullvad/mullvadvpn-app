import React, { createContext, ReactNode, useContext } from 'react';

type InputOptionContextType = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  labelId: string | undefined;
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

export const useInputOption = (): InputOptionContextType => {
  const context = useContext(InputOptionContextContext);
  if (!context) {
    throw new Error('useInputOption must be used within an InputOptionProvider');
  }
  return context;
};
