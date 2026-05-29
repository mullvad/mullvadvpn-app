import React from 'react';
import styled from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { Popup } from '../../../popup';
import { useMenuContext } from '../../MenuContext';

export type MenuPopupProps = React.ComponentPropsWithoutRef<'dialog'>;

export const StyledMenuPopup = styled(Popup).attrs<{ $popupId: string }>(({ $popupId }) => ({
  // Set via attrs function to avoid generating a class for each instance of the popup
  style: {
    positionAnchor: `--${$popupId}`,
  } as React.CSSProperties,
}))<{
  $popupId: string;
}>`
  --transition-duration: 0.1s;
  --initial-opacity: 0;
  --initial-scale: 0.9;

  inset: auto;

  position-try-fallbacks: flip-block, flip-inline;
  top: calc(anchor(bottom) + ${spacings.tiny});
  right: anchor(center);

  opacity: var(--initial-opacity);
  scale: var(--initial-scale);

  box-sizing: border-box;
  background: ${colors.blue40};
  border-radius: ${Radius.radius4};
  border: 2px solid ${colors.darkBlue};
  padding: 6px ${spacings.tiny};
  max-width: 65vw;
  max-height: calc(50vh - ${spacings.medium});

  // Make the popup scrollable if content exceeds max height
  // but hide the scrollbar
  overflow-y: scroll;
  &::-webkit-scrollbar {
    display: none;
  }

  box-shadow:
    0 2px 8px rgba(0, 0, 0, 0.1),
    0 8px 12px rgba(0, 0, 0, 0.1);

  transition-property: opacity, scale, display, overlay;
  transition-duration: var(--transition-duration);
  transition-behavior: allow-discrete;

  &:open {
    opacity: 1;
    scale: 1;

    @starting-style {
      opacity: var(--initial-opacity);
      scale: var(--initial-scale);
    }
  }

  &::backdrop {
    background: transparent;
  }
`;

export function MenuPopup({ children, ...props }: MenuPopupProps) {
  const { open, onOpenChange, popupId } = useMenuContext();

  return (
    <StyledMenuPopup
      $popupId={popupId}
      id={popupId}
      tabIndex={-1}
      open={open}
      onOpenChange={onOpenChange}
      {...props}>
      {children}
    </StyledMenuPopup>
  );
}
