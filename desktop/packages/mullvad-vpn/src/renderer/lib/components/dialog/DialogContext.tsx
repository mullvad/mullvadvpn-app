import React from 'react';

type DialogContextProps = Omit<DialogProviderProps, 'children'> & {
  titleId: string;
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
}>;

export function DialogProvider({ children, ...props }: DialogProviderProps) {
  const titleId = React.useId();

  const value = React.useMemo(
    () => ({
      titleId,
      ...props,
    }),
    [titleId, props],
  );

  return <DialogContext.Provider value={value}>{children}</DialogContext.Provider>;
}
