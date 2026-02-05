import styled from 'styled-components';

import { Radius } from '../../../../foundations';
import { FlexRow } from '../../../flex-row';
import { StyledListItemTrailingAction } from '../list-item-trailing-action/ListItemTrailingAction';

export type ListItemTrailingActionsProps = React.ComponentPropsWithRef<'div'>;

export const StyledListItemTrailingActions = styled(FlexRow)`
  // First action should be smaller when there are more than two actions
  &:has(> :nth-child(3)) > :first-child {
    width: 32px;
  }
  // Last action should have rounded corners
  & > :last-child ${StyledListItemTrailingAction} {
    margin-left: 2px;
    border-top-right-radius: ${Radius.radius16};
    border-bottom-right-radius: ${Radius.radius16};
  }
`;

export function ListItemTrailingActions({ children, ...props }: ListItemTrailingActionsProps) {
  return (
    <StyledListItemTrailingActions data-action-group {...props}>
      {children}
    </StyledListItemTrailingActions>
  );
}
