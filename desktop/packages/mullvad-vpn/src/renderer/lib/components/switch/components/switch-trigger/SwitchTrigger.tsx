import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { useSwitchContext } from '../switch-context';

export type SwitchTriggerProps = React.ComponentPropsWithRef<'button'>;

export const StyledSwitchTrigger = styled.button<{ $checked?: boolean }>`
  background-color: transparent;
  width: fit-content;

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: -1px;
  }
`;

export function SwitchTrigger(props: SwitchTriggerProps) {
  const { labelId, checked, disabled, onCheckedChange } = useSwitchContext();
  const handleClick = React.useCallback(() => {
    if (onCheckedChange) {
      onCheckedChange(!checked);
    }
  }, [checked, onCheckedChange]);

  return (
    <StyledSwitchTrigger
      onClick={handleClick}
      disabled={disabled}
      role="switch"
      aria-checked={checked ? 'true' : 'false'}
      aria-labelledby={labelId}
      {...props}
    />
  );
}
