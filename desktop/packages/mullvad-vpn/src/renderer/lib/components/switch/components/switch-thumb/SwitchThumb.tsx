import React from 'react';
import styled, { css } from 'styled-components';

import { colors } from '../../../../foundations';
import { Dot } from '../../../dot';
import { useSwitchContext } from '../switch-context';
import { useBackgroundColor, useBorderColor } from './hooks';

export type SwitchThumbProps = React.HtmlHTMLAttributes<HTMLDivElement>;

const StyledSwitchThumbIndicator = styled(Dot)<{
  $checked?: boolean;
  $backgroundColor?: string;
}>`
  ${({ $checked, $backgroundColor }) => {
    return css`
      position: absolute;
      left: 2px;
      background-color: ${$backgroundColor};
      transform: translateX(${$checked ? '11px' : '1px'});
      transition:
        width 150ms ease,
        height 150ms ease,
        transform 150ms ease,
        background-color 100ms linear;
    `;
  }}
`;

const StyledSwitchThumbTrack = styled.div<{ $borderColor: string }>`
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

      &:focus-visible {
        outline: 2px solid ${colors.white};
        outline-offset: 2px;
      }
    `;
  }}
`;

export function SwitchThumb(props: SwitchThumbProps) {
  const { checked } = useSwitchContext();
  const backgroundColor = useBackgroundColor();
  const borderColor = useBorderColor();
  return (
    <StyledSwitchThumbTrack $borderColor={colors[borderColor]} {...props}>
      <StyledSwitchThumbIndicator $checked={checked} $backgroundColor={colors[backgroundColor]} />
    </StyledSwitchThumbTrack>
  );
}
