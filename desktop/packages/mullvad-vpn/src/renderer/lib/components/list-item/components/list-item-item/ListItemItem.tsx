import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import { useListItemAnimation } from '../../hooks';
import { useListItemContext } from '../../ListItemContext';
import { useBackgroundColor } from './hooks';

export type ListItemItemProps = {
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

export const StyledListItemItem = styled.div<{
  $backgroundColor: string;
  $animation?: RuleSet<object>;
}>`
  ${({ $backgroundColor, $animation }) => {
    return css`
      --background-color: ${$backgroundColor};

      margin-bottom: 1px;
      background-color: var(--background-color);
      min-height: 48px;
      width: 100%;
      display: grid;
      grid-template-columns: 1fr;
      background-color: var(--background-color);
      &&:has(> :last-child:nth-child(2)) {
        grid-template-columns: 1fr 56px;
      }
      ${$animation}
    `;
  }}
`;

export function ListItemItem({ children, ...props }: ListItemItemProps) {
  const backgroundColor = useBackgroundColor();
  const { animation: contextAnimation } = useListItemContext();
  const animation = useListItemAnimation(contextAnimation);
  return (
    <StyledListItemItem
      $backgroundColor={backgroundColor}
      $animation={contextAnimation === 'flash' ? animation : undefined}
      {...props}>
      {children}
    </StyledListItemItem>
  );
}
