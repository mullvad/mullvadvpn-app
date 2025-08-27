import React from 'react';
interface FeatureIndicatorContextProps {
  disabled?: boolean;
}
const FeatureIndicatorContext = React.createContext<FeatureIndicatorContextProps | undefined>(
  undefined,
);

export const useFeatureIndicatorContext = (): FeatureIndicatorContextProps => {
  const context = React.useContext(FeatureIndicatorContext);
  if (!context) {
    throw new Error('useFeatureIndicatorContext must be used within a FeatureIndicatorProvider');
  }
  return context;
};

interface FeatureIndicatorProviderProps {
  disabled?: boolean;
  children: React.ReactNode;
}

export const FeatureIndicatorProvider = ({ disabled, children }: FeatureIndicatorProviderProps) => {
  return (
    <FeatureIndicatorContext.Provider value={{ disabled }}>
      {children}
    </FeatureIndicatorContext.Provider>
  );
};
