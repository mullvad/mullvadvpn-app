import React from 'react';

import type { DialogProps } from '../../../../../lib/components/dialog';
import { useTextField, type UseTextFieldState } from '../../../../../lib/components/text-field';
import type { CustomListLocation } from '../../select-location-types';
import { useIsUpdatedCustomListNameValid } from './hooks';

type EditListDialogContext = Omit<EditListDialogProviderProps, 'children'> & {
  formRef: React.RefObject<HTMLFormElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  form: {
    error: boolean;
    setError: React.Dispatch<React.SetStateAction<boolean>>;
    customListTextField: UseTextFieldState;
  };
};

const EditListDialogContext = React.createContext<EditListDialogContext | undefined>(undefined);

export const useEditListDialogContext = (): EditListDialogContext => {
  const context = React.useContext(EditListDialogContext);
  if (!context) {
    throw new Error('useEditListDialogContext must be used within a EditListDialogProvider');
  }
  return context;
};

type EditListDialogProviderProps = React.PropsWithChildren<DialogProps> & {
  customList: CustomListLocation;
};

export function EditListDialogProvider({
  customList,
  children,
  ...props
}: EditListDialogProviderProps) {
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

  return <EditListDialogContext.Provider value={value}>{children}</EditListDialogContext.Provider>;
}
