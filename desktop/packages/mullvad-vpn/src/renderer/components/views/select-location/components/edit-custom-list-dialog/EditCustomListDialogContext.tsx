import React from 'react';

import type { DialogProps } from '../../../../../lib/components/dialog';
import { useTextField, type UseTextFieldState } from '../../../../../lib/components/text-field';
import type { CustomListLocation } from '../../select-location-types';
import { useIsUpdatedCustomListNameValid } from './hooks';

type EditCustomListDialogContextProps = Omit<EditCustomListDialogProviderProps, 'children'> & {
  formRef: React.RefObject<HTMLFormElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  form: {
    error: boolean;
    setError: React.Dispatch<React.SetStateAction<boolean>>;
    customListTextField: UseTextFieldState;
  };
};

const EditCustomListDialogContext = React.createContext<
  EditCustomListDialogContextProps | undefined
>(undefined);

export const useEditCustomListDialogContext = (): EditCustomListDialogContextProps => {
  const context = React.useContext(EditCustomListDialogContext);
  if (!context) {
    throw new Error(
      'useEditCustomListDialogContext must be used within a EditCustomListDialogProvider',
    );
  }
  return context;
};

type EditCustomListDialogProviderProps = Pick<DialogProps, 'open' | 'onOpenChange' | 'children'> & {
  customList: CustomListLocation;
};

export function EditCustomListDialogProvider({
  customList,
  children,
  ...props
}: EditCustomListDialogProviderProps) {
  const formRef = React.useRef<HTMLFormElement>(null);
  const inputRef = React.useRef<HTMLInputElement>(null);
  const [error, setError] = React.useState<boolean>(false);
  const isCustomListNameValid = useIsUpdatedCustomListNameValid();

  const customListTextField = useTextField({
    defaultValue: customList.label,
    inputRef,
    validate: isCustomListNameValid,
  });

  const value = React.useMemo(
    () => ({
      formRef,
      inputRef,
      form: {
        error,
        setError,
        customListTextField,
      },
      customList,
      ...props,
    }),
    [customListTextField, error, props, customList],
  );

  return (
    <EditCustomListDialogContext.Provider value={value}>
      {children}
    </EditCustomListDialogContext.Provider>
  );
}
