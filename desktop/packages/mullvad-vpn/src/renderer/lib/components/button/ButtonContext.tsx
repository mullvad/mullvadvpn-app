import React from 'react';

interface ButtonContextProps {
  disabled: boolean;
}

const ButtonContext = React.createContext<ButtonContextProps | undefined>(undefined);

export const useButtonContext = (): ButtonContextProps => {
  const context = React.useContext(ButtonContext);
  if (!context) {
    throw new Error('useButtonContext must be used within a ButtonProvider');
  }
  return context;
};

interface ButtonProviderProps {
  disabled: boolean;
  children: React.ReactNode;
}

export const ButtonProvider = ({ disabled, children }: ButtonProviderProps) => {
  return <ButtonContext.Provider value={{ disabled }}>{children}</ButtonContext.Provider>;
};
