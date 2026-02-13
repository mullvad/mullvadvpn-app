import React from 'react';

import { Alert } from '../alert';
import { Button } from '../button';
import {
  DialogButtonGroup,
  DialogCloseButton,
  DialogPopup,
  DialogPopupContent,
  DialogPortal,
  DialogTitle,
} from './components';
import { DialogProvider } from './DialogContext';

export type DialogProps = React.PropsWithChildren<{
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
}>;

function Dialog({ open, onOpenChange, children }: DialogProps) {
  const [mounted, setMounted] = React.useState(open);

  React.useEffect(() => {
    if (open) {
      setMounted(true);
    }
  }, [open]);

  if (!mounted) return null;

  return (
    <DialogProvider
      open={open}
      onOpenChange={onOpenChange}
      mounted={mounted}
      setMounted={setMounted}>
      {children}
    </DialogProvider>
  );
}

const DialogNamespace = Object.assign(Dialog, {
  Popup: DialogPopup,
  PopupContent: DialogPopupContent,
  ButtonGroup: DialogButtonGroup,
  TextGroup: Alert.TextGroup,
  Text: Alert.Text,
  Title: DialogTitle,
  Icon: Alert.Icon,
  Button: Button,
  CloseButton: DialogCloseButton,
  Portal: DialogPortal,
});

export { DialogNamespace as Dialog };
