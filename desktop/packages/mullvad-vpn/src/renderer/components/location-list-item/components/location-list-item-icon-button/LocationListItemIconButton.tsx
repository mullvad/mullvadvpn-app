import React from 'react';
import styled from 'styled-components';

import { IconButton, type IconButtonProps } from '../../../../lib/components';

export type LocationListItemIconButtonProps = IconButtonProps;

export const StyledLocationListItemIconButton = styled(IconButton)``;

function LocationListItemIconButton({
  children,
  onClick,
  ...props
}: LocationListItemIconButtonProps) {
  const handleClick = React.useCallback(
    (event: React.MouseEvent<HTMLButtonElement, MouseEvent>) => {
      // Prevent click from propagating to list item trigger
      event.stopPropagation();
      onClick?.(event);
    },
    [onClick],
  );

  const handleKeydown = React.useCallback((event: React.KeyboardEvent<HTMLButtonElement>) => {
    // Prevent keydown from propagating to list item trigger
    event.stopPropagation();
    if (event.key === 'Enter' || event.key === ' ') {
      if (event.key === 'Enter' || event.key === ' ') {
        if (event.key === ' ') {
          event.preventDefault();
        }

        event.currentTarget.click();
      }
    }
  }, []);

  return (
    <StyledLocationListItemIconButton
      onClick={handleClick}
      onKeyDown={handleKeydown}
      variant="secondary"
      {...props}>
      {children}
    </StyledLocationListItemIconButton>
  );
}

const LocationListItemIconButtonNamespace = Object.assign(LocationListItemIconButton, {
  Icon: IconButton.Icon,
});

export { LocationListItemIconButtonNamespace as LocationListItemIconButton };
