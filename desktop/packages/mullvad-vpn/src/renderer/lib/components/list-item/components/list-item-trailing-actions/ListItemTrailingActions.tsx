import styled from 'styled-components';

import { Radius } from '../../../../foundations';
import { FlexRow } from '../../../flex-row';
import { ListItemTrailingAction, StyledListItemTrailingAction } from './components';

export type ListItemTrailingActionsProps = React.ComponentPropsWithRef<'div'>;

export const StyledListItemTrailingActions = styled(FlexRow)`
  // First action should be smaller when there are more than two actions
  &:has(> :nth-child(3)) > :first-child {
    width: 32px;
  }

  // Add border radius to last action
  & > ${StyledListItemTrailingAction}:last-child {
    border-top-right-radius: ${Radius.radius16};
    border-bottom-right-radius: ${Radius.radius16};
  }

  // If last action is nested inside a trigger, add margin and border radius
  & > :last-child ${StyledListItemTrailingAction} {
    margin-left: 2px;
    border-top-right-radius: ${Radius.radius16};
    border-bottom-right-radius: ${Radius.radius16};
  }
`;

function ListItemTrailingActions({ children, ...props }: ListItemTrailingActionsProps) {
  return <StyledListItemTrailingActions {...props}>{children}</StyledListItemTrailingActions>;
}

const ListItemTrailingActionsNamespace = Object.assign(ListItemTrailingActions, {
  Action: ListItemTrailingAction,
});

export { ListItemTrailingActionsNamespace as ListItemTrailingActions };
