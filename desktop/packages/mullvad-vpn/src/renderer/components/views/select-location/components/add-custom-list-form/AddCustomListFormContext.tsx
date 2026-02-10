import React from 'react';

import { useTextField, type UseTextFieldState } from '../../../../../lib/components/text-field';
import { useIsCustomListNameValid } from './hooks';

type AddCustomListFormContext = Omit<AddCustomListFormProviderProps, 'children'> & {
  formRef: React.RefObject<HTMLFormElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  form: {
    error: boolean;
    setError: React.Dispatch<React.SetStateAction<boolean>>;
    customListTextField: UseTextFieldState;
  };
};

const AddCustomListFormContext = React.createContext<AddCustomListFormContext | undefined>(
  undefined,
);

export const useAddCustomListFormContext = (): AddCustomListFormContext => {
  const context = React.useContext(AddCustomListFormContext);
  if (!context) {
    throw new Error('useAddCustomListFormContext must be used within a AddCustomListFormProvider');
  }
  return context;
};

type AddCustomListFormProviderProps = React.PropsWithChildren;

export function AddCustomListFormProvider({ children, ...props }: AddCustomListFormProviderProps) {
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
    <AddCustomListFormContext.Provider value={value}>{children}</AddCustomListFormContext.Provider>
  );
}
