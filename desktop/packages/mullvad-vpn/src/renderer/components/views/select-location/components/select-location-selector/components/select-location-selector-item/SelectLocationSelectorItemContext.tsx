import React from 'react';

import { type LocationSelectorSelectedItem } from '../../../../../../../lib/components/location-selector';
import {
  useTextField,
  type UseTextFieldState,
} from '../../../../../../../lib/components/text-field';
import { useSelectLocationViewContext } from '../../../../SelectLocationViewContext';

type SelectLocationSelectorItemContextProps = Omit<
  SelectLocationSelectorItemProviderProps,
  'children'
> & {
  triggerRef: React.RefObject<HTMLDivElement | null>;
  textField: UseTextFieldState;
  focused: boolean;
  setFocused: React.Dispatch<React.SetStateAction<boolean>>;
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
  id: LocationSelectorSelectedItem;
  defaultValue?: string;
  delay?: number;
  triggerRef?: React.RefObject<HTMLDivElement | null>;
}>;

export function SelectLocationSelectorItemProvider({
  id,
  defaultValue,
  delay = 0,
  triggerRef: triggerRefProp,
  children,
}: SelectLocationSelectorItemProviderProps) {
  const inputRef = React.useRef<HTMLInputElement | null>(null);
  const triggerRef = React.useRef<HTMLDivElement | null>(null);
  const [focused, setFocused] = React.useState(false);
  const { searchTerm } = useSelectLocationViewContext();

  const {
    handleOnValueChange,
    handleFocus: handleFocusTextField,
    handleBlur: handleBlurTextField,
    reset,
    value: valueTextField,
    ...textField
  } = useTextField({
    defaultValue,
    inputRef,
    delay,
  });

  const handleFocus = React.useCallback(() => {
    handleFocusTextField();
    setFocused(true);

    if (inputRef.current) {
      // Select all text
      inputRef.current.select();
    }
  }, [handleFocusTextField]);

  const handleBlur = React.useCallback(() => {
    handleBlurTextField();

    // Clear the selection to ensure we can re-select the text next time the input gets focused.
    // Note that this clears every selection across the entire page, but seeing as the input
    // element is the only thing that can be focused right now, in effect it only removes
    // the selection for the input field.
    window.getSelection()?.removeAllRanges();
  }, [handleBlurTextField]);

  // If selected location is changed, update text field value to match the selected location.
  React.useEffect(() => {
    if (!focused && !searchTerm) {
      handleOnValueChange(defaultValue ?? '');
    }
  }, [defaultValue, focused, handleOnValueChange, searchTerm]);

  const value = React.useMemo(
    () => ({
      textField: {
        handleOnValueChange,
        handleFocus,
        handleBlur,
        reset,
        value: valueTextField,
        ...textField,
      },
      id,
      focused,
      setFocused,
      triggerRef: triggerRefProp ?? triggerRef,
    }),
    [
      handleOnValueChange,
      handleFocus,
      handleBlur,
      reset,
      valueTextField,
      textField,
      id,
      focused,
      triggerRefProp,
    ],
  );

  return (
    <SelectLocationSelectorItemContext.Provider value={value}>
      {children}
    </SelectLocationSelectorItemContext.Provider>
  );
}
