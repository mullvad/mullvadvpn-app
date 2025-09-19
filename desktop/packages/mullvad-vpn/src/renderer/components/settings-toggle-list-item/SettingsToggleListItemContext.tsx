import { createContext, useContext } from 'react';

type SettingsToggleListItemContextType = {
  descriptionId: string;
};

const SettingsToggleListItemContext = createContext<SettingsToggleListItemContextType | undefined>(
  undefined,
);

type SettingsToggleListItemProviderProps =
  React.PropsWithChildren<SettingsToggleListItemContextType>;

export const SettingsToggleListItemProvider = ({
  children,
  ...props
}: SettingsToggleListItemProviderProps) => {
  return (
    <SettingsToggleListItemContext.Provider value={props}>
      {children}
    </SettingsToggleListItemContext.Provider>
  );
};

export const useSettingsToggleListItemContext = (): SettingsToggleListItemContextType => {
  const context = useContext(SettingsToggleListItemContext);
  if (!context) {
    throw new Error(
      'useSettingsToggleListItem must be used within a SettingsToggleListItemProvider',
    );
  }
  return context;
};
