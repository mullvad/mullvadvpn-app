import React from 'react';
import styled, { css, RuleSet } from 'styled-components';

import { Radius } from '../../../../foundations';
import { FlexRow } from '../../../flex-row';
import { useListItemAnimation, useListItemBackgroundColor } from '../../hooks';
import { useListItemContext } from '../../ListItemContext';
import { useIndent } from './hooks';

export type ListItemItemProps = {
  children: React.ReactNode;
} & React.ComponentPropsWithRef<'div'>;

export const StyledListItemItem = styled(FlexRow)<{
  $backgroundColor: string;
  $paddingLeft: string;
  $animation?: RuleSet<object>;
}>`
  ${({ $backgroundColor, $paddingLeft, $animation }) => {
    return css`
      --background-color: ${$backgroundColor};

      min-height: 48px;
      grid-template-columns: 1fr auto;
      background-color: var(--background-color);
      border-radius: ${Radius.radius16};

      > :first-child {
        padding-left: ${$paddingLeft};
      }

      ${$animation}
    `;
  }}
`;

export function ListItemItem({ children, ...props }: ListItemItemProps) {
  const backgroundColor = useListItemBackgroundColor();
  const { animation: contextAnimation } = useListItemContext();
  const animation = useListItemAnimation(contextAnimation);
  const paddingLeft = useIndent();
  return (
    <StyledListItemItem
      alignItems="center"
      justifyContent="space-between"
      flexGrow={1}
      gap="small"
      $paddingLeft={paddingLeft}
      $backgroundColor={backgroundColor}
      $animation={contextAnimation === 'flash' ? animation : undefined}
      {...props}>
      {children}
    </StyledListItemItem>
  );
}
