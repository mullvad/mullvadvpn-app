import styled, { css } from 'styled-components';

import { useBackgroundColor } from './hooks';

export interface ListItemItemProps {
  children: React.ReactNode;
}

const StyledDiv = styled.div<{ $backgroundColor: string }>`
  ${({ $backgroundColor }) => {
    return css`
      --background-color: ${$backgroundColor};
      background-color: var(--background-color);
      min-height: 44px;
      width: 100%;
      display: grid;
      grid-template-columns: 1fr;
      background-color: var(--background-color);
      &&:has(> :last-child:nth-child(2)) {
        grid-template-columns: 1fr 56px;
      }
    `;
  }}
`;

export function ListItemItem({ children }: ListItemItemProps) {
  const backgroundColor = useBackgroundColor();
  return <StyledDiv $backgroundColor={backgroundColor}>{children}</StyledDiv>;
}
