import React from 'react';

interface AnimateContextProps {
  present?: boolean;
  initial?: boolean;
  show: boolean;
  setShow: React.Dispatch<React.SetStateAction<boolean>>;
}

const AnimateContext = React.createContext<AnimateContextProps | undefined>(undefined);

export const useAnimateContext = (): AnimateContextProps => {
  const context = React.useContext(AnimateContext);
  if (!context) {
    throw new Error('useButtonContext must be used within a ButtonProvider');
  }
  return context;
};

interface AnimateProviderProps {
  present?: boolean;
  initial?: boolean;
  children: React.ReactNode;
}

export const AnimateProvider = ({ present, initial, children }: AnimateProviderProps) => {
  const [show, setShow] = React.useState<boolean>(present || false);
  return (
    <AnimateContext.Provider value={{ present, initial, show, setShow }}>
      {children}
    </AnimateContext.Provider>
  );
};
