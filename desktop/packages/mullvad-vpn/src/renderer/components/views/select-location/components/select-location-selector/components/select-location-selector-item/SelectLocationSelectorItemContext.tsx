import React from 'react';

import {
  useTextField,
  type UseTextFieldState,
} from '../../../../../../../lib/components/text-field';

type SelectLocationSelectorItemContextProps = Omit<
  SelectLocationSelectorItemProviderProps,
  'children'
> & {
  triggerRef: React.RefObject<HTMLDivElement | null>;
  textField: UseTextFieldState;
  searching: boolean;
  setSearching: React.Dispatch<React.SetStateAction<boolean>>;
};

const SelectLocationSelectorItemContext = React.createContext<
  SelectLocationSelectorItemContextProps | undefined
>(undefined);

export const useSelectLocationSelectorItemContext = (): SelectLocationSelectorItemContextProps => {
  const context = React.useContext(SelectLocationSelectorItemContext);
  if (!context) {
    throw new Error(
      'useSelectLocationSelectorItemContext must be used within a SelectLocationSelectorItemProvider',
    );
  }
  return context;
};

type SelectLocationSelectorItemProviderProps = React.PropsWithChildren<{
  defaultValue?: string;
  delay?: number;
  triggerRef?: React.RefObject<HTMLDivElement | null>;
}>;

export function SelectLocationSelectorItemProvider({
  defaultValue,
  delay = 200,
  triggerRef: triggerRefProp,
  children,
}: SelectLocationSelectorItemProviderProps) {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const triggerRef = React.useRef<HTMLDivElement | null>(null);
  const [searching, setSearching] = React.useState(false);

  const { handleOnValueChange, ...textField } = useTextField({
    defaultValue,
    inputRef,
    delay,
  });

  // If selected location is changed, update text field value to match the new selected location.
  React.useEffect(() => {
    if (!searching) {
      handleOnValueChange(defaultValue ?? '');
    }
  }, [defaultValue, handleOnValueChange, searching]);

  const value = React.useMemo(
    () => ({
      textField: {
        handleOnValueChange,
        ...textField,
      },
      searching,
      setSearching,
      triggerRef: triggerRefProp ? triggerRefProp : triggerRef,
    }),
    [handleOnValueChange, textField, searching, triggerRefProp],
  );

  return (
    <SelectLocationSelectorItemContext.Provider value={value}>
      {children}
    </SelectLocationSelectorItemContext.Provider>
  );
}
