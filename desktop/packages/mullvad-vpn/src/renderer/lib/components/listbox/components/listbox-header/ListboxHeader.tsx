import styled from 'styled-components';

import { ListItem, ListItemProps } from '../../../list-item';

export type ListboxHeaderProps = ListItemProps;

export const StyledListboxHeader = styled(ListItem)``;

export function ListboxHeader({ children, ...props }: ListboxHeaderProps) {
  return (
    <StyledListboxHeader position="first" {...props}>
      {children}
    </StyledListboxHeader>
  );
}
