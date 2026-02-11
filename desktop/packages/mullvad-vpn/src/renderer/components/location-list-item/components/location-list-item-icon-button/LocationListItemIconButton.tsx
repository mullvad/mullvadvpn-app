import styled from 'styled-components';

import { IconButton, type IconButtonProps } from '../../../../lib/components';
import { useListItemContext } from '../../../../lib/components/list-item/ListItemContext';

export type LocationListItemIconButtonProps = IconButtonProps;

export const StyledLocationListItemIconButton = styled(IconButton)``;

function LocationListItemIconButton({ children, ...props }: LocationListItemIconButtonProps) {
  const { disabled } = useListItemContext();
  return (
    <StyledLocationListItemIconButton variant="secondary" disabled={disabled} {...props}>
      {children}
    </StyledLocationListItemIconButton>
  );
}

const LocationListItemIconButtonNamespace = Object.assign(LocationListItemIconButton, {
  Icon: IconButton.Icon,
});

export { LocationListItemIconButtonNamespace as LocationListItemIconButton };
