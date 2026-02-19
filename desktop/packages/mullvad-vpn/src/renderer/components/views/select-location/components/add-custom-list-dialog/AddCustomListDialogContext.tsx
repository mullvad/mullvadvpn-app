import React from 'react';

import type { DialogProps } from '../../../../../lib/components/dialog';
import { useTextField, type UseTextFieldState } from '../../../../../lib/components/text-field';
import { useIsCustomListNameValid } from './hooks';

type AddCustomListDialogContextProps = Omit<AddCustomListDialogProviderProps, 'children'> & {
  formRef: React.RefObject<HTMLFormElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  form: {
    error: boolean;
    setError: React.Dispatch<React.SetStateAction<boolean>>;
    customListTextField: UseTextFieldState;
  };
};

const AddCustomListDialogContext = React.createContext<AddCustomListDialogContextProps | undefined>(
  undefined,
);

export const useAddCustomListDialogContext = (): AddCustomListDialogContextProps => {
  const context = React.useContext(AddCustomListDialogContext);
  if (!context) {
    throw new Error(
      'useAddCustomListDialogContext must be used within a AddCustomListDialogProvider',
    );
  }
  return context;
};

type AddCustomListDialogProviderProps = Pick<DialogProps, 'open' | 'onOpenChange' | 'children'>;

export function AddCustomListDialogProvider({
  children,
  ...props
}: AddCustomListDialogProviderProps) {
  const formRef = React.useRef<HTMLFormElement>(null);
  const inputRef = React.useRef<HTMLInputElement>(null);
  const [error, setError] = React.useState<boolean>(false);
  const isCustomListNameValid = useIsCustomListNameValid();

  const customListTextField = useTextField({
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
      ...props,
    }),
    [customListTextField, error, props],
  );

  return (
    <AddCustomListDialogContext.Provider value={value}>
      {children}
    </AddCustomListDialogContext.Provider>
  );
}
