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
  CloseButton: DialogCloseButton,
  TextGroup: Alert.TextGroup,
  Text: Alert.Text,
  Title: DialogTitle,
  Subtitle: Alert.Subtitle,
  Icon: Alert.Icon,
  IconBadge: Alert.IconBadge,
  Spinner: Alert.Spinner,
  Button: Button,
  Portal: DialogPortal,
  List: Alert.List,
});

export { DialogNamespace as Dialog };
