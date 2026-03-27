import React from 'react';

import { useTextField, type UseTextFieldState } from '../../../../lib/components/text-field';
import { useIsCustomListNameValid } from '../../hooks';
import type { CreateCustomListDialogProps } from './CreateCustomListDialog';

type CreateCustomListDialogContextProps = Omit<CreateCustomListDialogProviderProps, 'children'> & {
  formRef: React.RefObject<HTMLFormElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  form: {
    error: boolean;
    setError: React.Dispatch<React.SetStateAction<boolean>>;
    customListTextField: UseTextFieldState;
  };
};

const CreateCustomListDialogContext = React.createContext<
  CreateCustomListDialogContextProps | undefined
>(undefined);

export const useCreateCustomListDialogContext = (): CreateCustomListDialogContextProps => {
  const context = React.useContext(CreateCustomListDialogContext);
  if (!context) {
    throw new Error(
      'useCreateCustomListDialogContext must be used within a CreateCustomListDialogProvider',
    );
  }
  return context;
};

type CreateCustomListDialogProviderProps = React.PropsWithChildren<
  Pick<
    CreateCustomListDialogProps,
    'open' | 'onOpenChange' | 'loading' | 'onLoadingChange' | 'location'
  >
>;

export function CreateCustomListDialogProvider({
  children,
  ...props
}: CreateCustomListDialogProviderProps) {
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
    <CreateCustomListDialogContext.Provider value={value}>
      {children}
    </CreateCustomListDialogContext.Provider>
  );
}
