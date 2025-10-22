import React from 'react';

import { Button, ButtonProps } from '../../../button';
import { useDialogContext } from '../../DialogContext';

export type DialogCloseButtonProps = ButtonProps;

export function DialogCloseButton({ children, ...props }: DialogCloseButtonProps) {
  const { dialogRef } = useDialogContext();

  const handleClick = React.useCallback(() => {
    const dialog = dialogRef.current;
    dialog?.blur();
    dialog?.requestClose();
  }, [dialogRef]);

  return (
    <Button onClick={handleClick} {...props}>
      {children}
    </Button>
  );
}
