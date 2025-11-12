import React, { useState } from 'react';

import { messages } from '../../shared/gettext';
import { Button, IconButton, IconButtonProps } from '../lib/components';
import { Dialog } from '../lib/components/dialog';

export interface InfoButtonProps extends Omit<IconButtonProps, 'icon'> {
  title?: string;
  message?: string | Array<string>;
  children?: React.ReactNode;
}

export default function InfoButton({ title, message, children, ...props }: InfoButtonProps) {
  const [open, setOpen] = useState(false);
  const show = React.useCallback(() => setOpen(true), []);
  const hide = React.useCallback(() => setOpen(false), []);

  return (
    <>
      <IconButton
        onClick={show}
        aria-label={messages.pgettext('accessibility', 'More information')}
        {...props}>
        <IconButton.Icon icon="info-circle" />
      </IconButton>
      <Dialog open={open} onOpenChange={setOpen}>
        <Dialog.Container>
          <Dialog.Icon icon="info-circle" />
          {children}
          <Button key="back" onClick={hide}>
            <Button.Text>{messages.gettext('Got it!')}</Button.Text>
          </Button>
        </Dialog.Container>
      </Dialog>
    </>
  );
}
