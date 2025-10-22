import React from 'react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../foundations';
import { Alert } from '../alert';
import { Button } from '../button';
import { DialogButtonGroup, DialogTitle } from './components';
import { DialogProvider, useDialogContext } from './DialogContext';
import { useAddEventListeners, useHandleClick, useHandleKeydown, useOpenSync } from './hooks';

export type DialogProps = React.ComponentPropsWithoutRef<'dialog'> & {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
};

const StyledDialog = styled.dialog`
  // Reset default dialog styles
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  max-width: none;
  user-select: none;

  top: 50%;
  left: 50%;
  translate: -50% -50%;

  border-radius: ${Radius.radius12};

  opacity: 0;
  scale: 0.9;
  transform-origin: center center;

  width: min(100vw, calc(100vw - (${spacings.medium} * 2)));

  transition-property: opacity, scale, display;
  transition-duration: 0.1s;
  transition-behavior: allow-discrete;

  &&[open] {
    display: block;
    opacity: 1;
    scale: 1;

    @starting-style {
      opacity: 0;
      scale: 0.9;
    }
  }

  &&::backdrop {
    opacity: 0;
    transition-property: opacity, display, overlay;
    transition-duration: 0.1s;
    transition-behavior: allow-discrete;
  }

  &&[open]::backdrop {
    opacity: 1;
    background: ${colors.blackAlpha50};
    backdrop-filter: blur(1.5px);
  }

  @starting-style {
    &&[open]::backdrop {
      opacity: 0;
    }
  }
`;

function DialogImpl({ children, ...props }: Omit<DialogProps, 'open' | 'onOpenChange'>) {
  const { titleId, dialogRef } = useDialogContext();

  useAddEventListeners();
  useOpenSync();

  const handleKeydown = useHandleKeydown();
  const handleClick = useHandleClick();

  return (
    <StyledDialog
      ref={dialogRef}
      onKeyDown={handleKeydown}
      onClick={handleClick}
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
});

export { DialogNamespace as Dialog };
