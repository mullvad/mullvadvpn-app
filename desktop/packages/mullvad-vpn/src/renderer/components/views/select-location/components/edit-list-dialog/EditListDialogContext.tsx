import React from 'react';

import type { DialogProps } from '../../../../../lib/components/dialog';
import { useTextField, type UseTextFieldState } from '../../../../../lib/components/text-field';
import type { CustomListSpecification } from '../../select-location-types';
import { useLocationRowContext } from '../location-row/LocationRowContext';
import { useIsUpdatedCustomListNameValid } from './hooks';

type EditListDialogContext = Omit<EditListDialogProviderProps, 'children'> & {
  source: CustomListSpecification;
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

type EditListDialogProviderProps = React.PropsWithChildren<DialogProps>;

export function EditListDialogProvider({ children, ...props }: EditListDialogProviderProps) {
  const formRef = React.useRef<HTMLFormElement>(null);
  const inputRef = React.useRef<HTMLInputElement>(null);
  const [error, setError] = React.useState<boolean>(false);
  const isCustomListNameValid = useIsUpdatedCustomListNameValid();
  const { source: contextSource } = useLocationRowContext();

  const source: CustomListSpecification = React.useMemo(() => {
    if ('list' in contextSource) {
      return contextSource;
    } else {
      throw new Error('EditListDialog must be used with a location that has a custom list');
    }
  }, [contextSource]);

  const customListTextField = useTextField({
    defaultValue: source.list.name,
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
      source,
      ...props,
    }),
    [customListTextField, error, props, source],
  );

  return <EditListDialogContext.Provider value={value}>{children}</EditListDialogContext.Provider>;
}
