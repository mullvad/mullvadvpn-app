import React from 'react';
import styled, { css } from 'styled-components';

import { colors, Radius, spacings } from '../../../../foundations';
import { FlexColumn } from '../../../flex-column';
import { useMenuContext } from '../../MenuContext';
import { useEffectHideOnOutsideClick, useEffectSyncOpen, useHideOnEscapeDown } from './hooks';

export type MenuPopupProps = React.ComponentPropsWithoutRef<'div'>;

export const StyledMenuPopup = styled(FlexColumn)<{ $popoverId: string }>`
  ${({ $popoverId }) => {
    return css`
      inset: auto;
      margin: 0;

      position-anchor: --${$popoverId};
      position-try-fallbacks: flip-block, flip-inline;
      top: calc(anchor(bottom) + ${spacings.tiny});
      right: anchor(center);

      box-sizing: border-box;
      background-color: ${colors.blue40};
      border-radius: ${Radius.radius4};
      border: 2px solid ${colors.darkBlue};
      padding: 6px ${spacings.tiny};
      max-width: calc(100% - ${spacings.medium} * 2);
      z-index: 10;
    `;
  }}
`;

export function MenuPopup({ children, ...props }: MenuPopupProps) {
  const { open, popoverRef, popoverId } = useMenuContext();
  useEffectSyncOpen();
  useEffectHideOnOutsideClick();

  const handleKeydown = useHideOnEscapeDown();

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
      {...props}>
      {children}
    </StyledMenuPopup>
  );
}
