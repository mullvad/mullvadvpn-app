import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { IconButton, type IconButtonProps } from '../../../../../lib/components';
import { HeaderMenu } from '../header-menu/HeaderMenu';

export type HeaderMenuIconButtonProps = IconButtonProps;

export function HeaderMenuIconButton(props: HeaderMenuIconButtonProps) {
  const [open, setOpen] = React.useState(false);
  const buttonRef = React.useRef<HTMLButtonElement>(null);

  const toggleMenu = React.useCallback(() => {
    setOpen((open) => !open);
  }, [setOpen]);

  return (
    <>
      <IconButton
        ref={buttonRef}
        variant="secondary"
        onClick={toggleMenu}
        aria-label={
          // TRANSLATORS: Label for button opening select location menu.
          messages.pgettext('accessibility', 'Open select location menu')
        }
        {...props}>
        <IconButton.Icon icon="more-horizontal-circle" />
      </IconButton>
      <HeaderMenu open={open} onOpenChange={toggleMenu} triggerRef={buttonRef} />
    </>
  );
}
