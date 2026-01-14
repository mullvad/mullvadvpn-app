import React from 'react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { Alert } from '../alert';
import { Button } from '../button';
import { DialogButtonGroup, DialogCloseButton, DialogTitle } from './components';
import { DialogProvider, useDialogContext } from './DialogContext';
import { useEffectSyncOpen, useHandleClick, useHandleClose } from './hooks';

export type DialogProps = React.ComponentPropsWithoutRef<'dialog'> & {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
};

const StyledDialog = styled.dialog`
  --transition-duration: 0.25s;
  --initial-opacity: 0;
  --initial-scale: 0.9;

  // Reset default dialog styles
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  max-width: none;
  user-select: none;

  display: none;
  top: 50%;
  left: 50%;
  translate: -50% -50%;

  border-radius: ${Radius.radius12};

  opacity: var(--initial-opacity);
  scale: var(--initial-scale);
  transform-origin: center center;

  width: min(100vw, calc(100vw - (${spacings.medium} * 2)));

  transition-property: opacity, scale, display, overlay;
  transition-duration: var(--transition-duration);
  transition-behavior: allow-discrete;

  &&[open] {
    display: block;
    opacity: 1;
    scale: 1;

    @starting-style {
      opacity: var(--initial-opacity);
      scale: var(--initial-scale);
    }
  }

  &&::backdrop {
    background: ${colors.blackAlpha50};
    backdrop-filter: blur(1.5px);
    opacity: var(--initial-opacity);

    transition-property: opacity;
    transition-duration: var(--transition-duration);
  }

  &&[open]::backdrop {
    opacity: 1;
  }

  // Set starting styles here since psuedo-elements cannot contain nested rules
  @starting-style {
    &&[open]::backdrop {
      opacity: var(--initial-opacity);
    }
  }
`;

function DialogImpl({ children, ...props }: Omit<DialogProps, 'open' | 'onOpenChange'>) {
  const { titleId, dialogRef } = useDialogContext();

  useEffectSyncOpen();

  const handleClick = useHandleClick();
  const handleClose = useHandleClose();

  return (
    <StyledDialog
      ref={dialogRef}
      onClick={handleClick}
      onClose={handleClose}
      aria-modal="true"
      aria-labelledby={titleId}
      {...props}>
      {children}
    </StyledDialog>
  );
}

function Dialog({ open, onOpenChange, ...props }: DialogProps) {
  return (
    <DialogProvider open={open} onOpenChange={onOpenChange}>
      <DialogImpl {...props} />
    </DialogProvider>
  );
}

const DialogNamespace = Object.assign(Dialog, {
  Container: Alert,
  ButtonGroup: DialogButtonGroup,
  TextGroup: Alert.TextGroup,
  Text: Alert.Text,
  Title: DialogTitle,
  Icon: Alert.Icon,
  Button: Button,
  CloseButton: DialogCloseButton,
});

export { DialogNamespace as Dialog };
