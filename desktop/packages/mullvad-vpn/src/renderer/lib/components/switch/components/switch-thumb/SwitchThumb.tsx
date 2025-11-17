import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { useSwitchContext } from '../../';
import { useBackgroundColor, useBorderColor } from './hooks';

export type SwitchThumbProps = React.HtmlHTMLAttributes<HTMLDivElement>;

export const StyledSwitchThumbIndicator = styled(Dot)<{
  $checked?: boolean;
  $backgroundColor?: string;
}>`
  ${({ $backgroundColor }) => {
    return css`
      position: absolute;
      left: 2px;
      background-color: ${$backgroundColor};
    `;
  }}
`;

export const StyledSwitchThumb = styled.div<{ $borderColor: string }>`
  ${({ $borderColor }) => {
    return css`
      position: relative;
      display: flex;
      align-items: center;
      width: 32px;
      height: 20px;
      border: 2px solid ${$borderColor};
      border-radius: 100px;
      transition: border-color 200ms ease;
    `;
  }}
`;

export function SwitchThumb(props: SwitchThumbProps) {
  const { checked } = useSwitchContext();
  const backgroundColor = useBackgroundColor();
  const borderColor = useBorderColor();
  return (
    <StyledSwitchThumb $borderColor={colors[borderColor]} {...props}>
      <StyledSwitchThumbIndicator $checked={checked} $backgroundColor={colors[backgroundColor]} />
    </StyledSwitchThumb>
  );
}
