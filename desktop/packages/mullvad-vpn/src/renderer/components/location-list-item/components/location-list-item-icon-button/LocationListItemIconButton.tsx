import styled from 'styled-components';

import { IconButton, type IconButtonProps } from '../../../../lib/components';

export type LocationListItemIconButtonProps = IconButtonProps;

export const StyledLocationListItemIconButton = styled(IconButton)``;

function LocationListItemIconButton({ children, ...props }: LocationListItemIconButtonProps) {
  return (
    <StyledLocationListItemIconButton variant="secondary" {...props}>
      {children}
    </StyledLocationListItemIconButton>
  );
}

const LocationListItemIconButtonNamespace = Object.assign(LocationListItemIconButton, {
  Icon: IconButton.Icon,
});

export { LocationListItemIconButtonNamespace as LocationListItemIconButton };
