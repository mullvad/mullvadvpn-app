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
  return (
    <DialogProvider open={open} onOpenChange={onOpenChange}>
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
