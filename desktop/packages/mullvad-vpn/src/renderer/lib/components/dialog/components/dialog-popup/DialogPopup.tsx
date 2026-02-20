import React from 'react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { useDialogContext } from '../../DialogContext';
import { useEffectSyncOpen, useHandleAnimationEnd, useHandleClick, useHandleClose } from './hooks';

export type DialogPopupProps = React.ComponentPropsWithoutRef<'dialog'>;

export const StyledDialogPopup = styled.dialog`
  --transition-duration: 0.25s;
  --initial-opacity: 0;
  --initial-scale: 0.9;

  // Reset default dialog styles
  margin: 0;
  border: 0;
  background: transparent;
  max-width: none;
  user-select: none;

  display: none;
  top: 50%;
  left: 50%;
  translate: -50% -50%;

  padding: ${spacings.medium};
  background-color: ${colors.darkBlue};
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

export function DialogPopup({ children, ...props }: DialogPopupProps) {
  const { titleId, dialogRef } = useDialogContext();

  useEffectSyncOpen();

  const handleClick = useHandleClick();
  const handleClose = useHandleClose();

  const handleAnimationEnd = useHandleAnimationEnd();

  return (
    <StyledDialogPopup
      ref={dialogRef}
      onClick={handleClick}
      onClose={handleClose}
      onTransitionEnd={handleAnimationEnd}
      aria-modal="true"
      aria-labelledby={titleId}
      {...props}>
      {children}
    </StyledDialogPopup>
  );
}
