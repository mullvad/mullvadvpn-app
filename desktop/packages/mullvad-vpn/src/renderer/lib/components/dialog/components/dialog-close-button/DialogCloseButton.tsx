import React from 'react';

import { Button, ButtonProps } from '../../../button';
import { usePopupContext } from '../../../popup/PopupContext';

export type DialogCloseButtonProps = ButtonProps;

function DialogCloseButton({ children, ...props }: DialogCloseButtonProps) {
  const { popupRef } = usePopupContext();

  const handleClick = React.useCallback(() => {
    const popup = popupRef.current;
    popup?.blur();
    popup?.requestClose();
  }, [popupRef]);

  return (
    <Button onClick={handleClick} {...props}>
      {children}
    </Button>
  );
}
const DialogCloseButtonNamespace = Object.assign(DialogCloseButton, {
  Text: Button.Text,
  Icon: Button.Icon,
});

export { DialogCloseButtonNamespace as DialogCloseButton };
