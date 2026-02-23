import React from 'react';

type DialogContextProps = Omit<DialogProviderProps, 'children'> & {
  titleId: string;
  dialogRef: React.RefObject<HTMLDialogElement | null>;
};

const DialogContext = React.createContext<DialogContextProps | undefined>(undefined);

export const useDialogContext = (): DialogContextProps => {
  const context = React.useContext(DialogContext);
  if (!context) {
    throw new Error('useDialogContext must be used within a DialogProvider');
  }
  return context;
};

type DialogProviderProps = React.PropsWithChildren<{
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  mounted?: boolean;
  setMounted: React.Dispatch<React.SetStateAction<boolean | undefined>>;
}>;

export function DialogProvider({ children, ...props }: DialogProviderProps) {
  const titleId = React.useId();
  const dialogRef = React.useRef<HTMLDialogElement>(null);

  const value = React.useMemo(
    () => ({
      titleId,
      dialogRef,
      ...props,
    }),
    [titleId, dialogRef, props],
  );

  return <DialogContext.Provider value={value}>{children}</DialogContext.Provider>;
}
