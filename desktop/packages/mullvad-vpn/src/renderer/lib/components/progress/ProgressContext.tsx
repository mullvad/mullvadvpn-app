import React, { createContext, ReactNode, useContext } from 'react';

interface ProgressContextType {
  value: number;
  min: number;
  max: number;
  percent: number;
  disabled?: boolean;
}

const ProgressContext = createContext<ProgressContextType | undefined>(undefined);

interface ProgressProviderProps extends ProgressContextType {
  children: ReactNode;
}

export const ProgressProvider: React.FC<ProgressProviderProps> = ({
  value,
  min,
  max,
  percent,
  disabled,
  children,
}) => {
  return (
    <ProgressContext.Provider value={{ min, max, percent, value, disabled }}>
      {children}
    </ProgressContext.Provider>
  );
};

export const useProgress = (): ProgressContextType => {
  const context = useContext(ProgressContext);
  if (!context) {
    throw new Error('useProgress must be used within a ProgressProvider');
  }
  return context;
};
