import React, { createContext, ReactNode, useContext } from 'react';

type InputListboxOptionContextType = {
  inputRef: React.RefObject<HTMLInputElement | null>;
  labelId: string | undefined;
};

const InputListboxOptionContextContext = createContext<InputListboxOptionContextType | undefined>(
  undefined,
);

type InputListboxOptionProviderProps = {
  children: ReactNode;
} & InputListboxOptionContextType;

export const InputListboxOptionProvider = ({
  children,
  ...props
}: InputListboxOptionProviderProps) => {
  return (
    <InputListboxOptionContextContext.Provider value={props}>
      {children}
    </InputListboxOptionContextContext.Provider>
  );
};

export const useInputListboxOption = (): InputListboxOptionContextType => {
  const context = useContext(InputListboxOptionContextContext);
  if (!context) {
    throw new Error('useInputListboxOption must be used within a InputListboxOptionProvider');
  }
  return context;
};
