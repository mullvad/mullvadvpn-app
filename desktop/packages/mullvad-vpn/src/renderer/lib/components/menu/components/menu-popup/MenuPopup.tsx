import React from 'react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { useMenuContext } from '../../MenuContext';
import {
  useEffectHideOnOutsideClick,
  useEffectSyncOpen,
  useHideOnEscapeDown,
  useUnmount,
} from './hooks';

export type MenuPopupProps = React.ComponentPropsWithoutRef<'div'>;

export const StyledMenuPopup = styled.div<{ $popoverId: string }>`
  ${({ $popoverId }) => {
    return css`
      --transition-duration: 0.1s;
      --initial-opacity: 0;
      --initial-scale: 0.9;

      // Display block allow transition end events to fire when popover is closed,
      // visibility is controlled by opacity and mounted state
      display: block;
      inset: auto;
      margin: 0;
      z-index: 10;

      position-anchor: --${$popoverId};
      position-try-fallbacks: flip-block, flip-inline;
      top: calc(anchor(bottom) + ${spacings.tiny});
      right: anchor(center);

      opacity: var(--initial-opacity);
      scale: var(--initial-scale);

      box-sizing: border-box;
      background-color: ${colors.blue40};
      border-radius: ${Radius.radius4};
      border: 2px solid ${colors.darkBlue};
      padding: 6px ${spacings.tiny};
      max-width: calc(100% - ${spacings.medium} * 2);

      box-shadow:
        0 2px 8px rgba(0, 0, 0, 0.1),
        0 8px 12px rgba(0, 0, 0, 0.1);

      transition-property: opacity, scale, display, overlay;
      transition-duration: var(--transition-duration);
      transition-behavior: allow-discrete;

      &:popover-open {
        opacity: 1;
        scale: 1;

        @starting-style {
          opacity: var(--initial-opacity);
          scale: var(--initial-scale);
        }
      }
    `;
  }}
`;

export function MenuPopup({ children, ...props }: MenuPopupProps) {
  const { open, popoverRef, popoverId } = useMenuContext();
  useEffectSyncOpen();
  useEffectHideOnOutsideClick();

  const handleKeydown = useHideOnEscapeDown();
  const handleTransitionEnd = useUnmount();

  React.useEffect(() => {
    if (open) {
      popoverRef.current?.focus();
    }
  }, [open, popoverRef]);

  return (
    <StyledMenuPopup
      $popoverId={popoverId}
      ref={popoverRef}
      id={popoverId}
      popover="manual"
      role="menu"
      tabIndex={-1}
      onKeyDown={handleKeydown}
      onTransitionEnd={handleTransitionEnd}
      {...props}>
      {children}
    </StyledMenuPopup>
  );
}
