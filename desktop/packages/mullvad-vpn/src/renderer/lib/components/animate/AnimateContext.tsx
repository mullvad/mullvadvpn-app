import React, { useState } from 'react';

import { Animation } from './types';

interface AnimateContextProps {
  animations: Animation[];
  animate: boolean;
  animatePresent: boolean;
  present: boolean;
  initial?: boolean;
  setAnimate: React.Dispatch<React.SetStateAction<boolean>>;
  setAnimatePresent: React.Dispatch<React.SetStateAction<boolean>>;
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
  animations: Animation[];
  present: boolean;
  initial?: boolean;
  children: React.ReactNode;
}

export const AnimateProvider = ({
  animations,
  present,
  initial,
  children,
}: AnimateProviderProps) => {
  const [animate, setAnimate] = React.useState<boolean>((initial && present) || false);
  const [animatePresent, setAnimatePresent] = useState<boolean>(present);

  return (
    <AnimateContext.Provider
      value={{
        animate,
        animatePresent,
        animations,
        initial,
        present,
        setAnimate,
        setAnimatePresent,
      }}>
      {children}
    </AnimateContext.Provider>
  );
};
