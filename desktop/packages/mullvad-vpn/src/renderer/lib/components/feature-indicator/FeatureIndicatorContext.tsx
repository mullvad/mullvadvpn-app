import React from 'react';
interface FeatureIndicatorContextProps {
  disabled?: boolean;
  inactive?: boolean;
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
  inactive?: boolean;
  children: React.ReactNode;
}

export const FeatureIndicatorProvider = ({
  disabled,
  inactive,
  children,
}: FeatureIndicatorProviderProps) => {
  const value = React.useMemo(() => ({ disabled, inactive }), [disabled, inactive]);

  return (
    <FeatureIndicatorContext.Provider value={value}>{children}</FeatureIndicatorContext.Provider>
  );
};
