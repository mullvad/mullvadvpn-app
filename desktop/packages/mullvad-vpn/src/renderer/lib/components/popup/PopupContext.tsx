import React from 'react';

type PopupContextProps = Omit<PopupProviderProps, 'children'> & {
  popupRef: React.RefObject<HTMLDialogElement | null>;
};

const PopupContext = React.createContext<PopupContextProps | undefined>(undefined);

export const usePopupContext = (): PopupContextProps => {
  const context = React.useContext(PopupContext);
  if (!context) {
    throw new Error('usePopupContext must be used within a PopupProvider');
  }
  return context;
};

type PopupProviderProps = React.PropsWithChildren<{
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
  mounted?: boolean;
  setMounted: React.Dispatch<React.SetStateAction<boolean | undefined>>;
}>;

export function PopupProvider({ children, ...props }: PopupProviderProps) {
  const popupRef = React.useRef<HTMLDialogElement>(null);

  const value = React.useMemo(
    () => ({
      popupRef,
      ...props,
    }),
    [popupRef, props],
  );

  return <PopupContext.Provider value={value}>{children}</PopupContext.Provider>;
}
