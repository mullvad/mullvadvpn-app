import styled, { css } from 'styled-components';

import { Radius } from '../../../../foundations';
import { useListItemBackgroundColor } from '../../hooks';
import { ListItemTrailingActionIcon } from './components';

export type ListItemTrailingActionProps = React.ComponentPropsWithRef<'div'>;

export const StyledListItemTrailingAction = styled.div<{ $backgroundColor: string }>`
  ${({ $backgroundColor }) => {
    return css`
      display: grid;
      place-items: center;
      width: 48px;

      border-top-right-radius: ${Radius.radius16};
      border-bottom-right-radius: ${Radius.radius16};

      margin-left: 1px;
      background-color: ${$backgroundColor};
    `;
  }}
`;

function ListItemTrailingAction({ children, ...props }: ListItemTrailingActionProps) {
  const backgroundColor = useListItemBackgroundColor();
  return (
    <StyledListItemTrailingAction $backgroundColor={backgroundColor} {...props}>
      {children}
    </StyledListItemTrailingAction>
  );
}

const ListItemTrailingActionNamespace = Object.assign(ListItemTrailingAction, {
  Icon: ListItemTrailingActionIcon,
});

export { ListItemTrailingActionNamespace as ListItemTrailingAction };
