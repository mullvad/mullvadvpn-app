import React from 'react';
import styled from 'styled-components';

import { useEffectSyncOpen, useHandleAnimationEnd, useHandleClick, useHandleClose } from './hooks';
import { PopupProvider, usePopupContext } from './PopupContext';

export type PopupProps = React.ComponentPropsWithoutRef<'dialog'> & {
  open?: boolean;
  onOpenChange?: (open: boolean) => void;
};

export const StyledPopup = styled.dialog`
  // Reset default dialog styles
  margin: 0;
  border: 0;
  background: transparent;
  max-width: none;
  user-select: none;

  z-index: 10;
`;

function PopupImpl({ children, ...props }: Omit<PopupProps, 'open' | 'onOpenChange'>) {
  const { popupRef } = usePopupContext();

  useEffectSyncOpen();

  const handleClick = useHandleClick();
  const handleClose = useHandleClose();

  const handleAnimationEnd = useHandleAnimationEnd();

  return (
    <StyledPopup
      ref={popupRef}
      onClick={handleClick}
      onClose={handleClose}
      onTransitionEnd={handleAnimationEnd}
      aria-modal="true"
      {...props}>
      {children}
    </StyledPopup>
  );
}

export function Popup({ open, onOpenChange, ...props }: PopupProps) {
  const [mounted, setMounted] = React.useState(open);

  React.useEffect(() => {
    if (open) {
      setMounted(true);
    }
  }, [open]);

  if (!mounted) return null;

  return (
    <PopupProvider
      open={open}
      onOpenChange={onOpenChange}
      mounted={mounted}
      setMounted={setMounted}>
      <PopupImpl {...props} />
    </PopupProvider>
  );
}
