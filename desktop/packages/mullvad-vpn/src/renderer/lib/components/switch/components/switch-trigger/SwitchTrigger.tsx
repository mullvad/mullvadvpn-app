import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { useSwitchContext } from '../../';

export type SwitchTriggerProps = React.ComponentPropsWithRef<'button'>;

export const StyledSwitchTrigger = styled.button`
  background-color: transparent;
  width: fit-content;

  &&:not(:disabled):hover {
    // Scale takes a unitless ratio, so we derive it from the 12px base: 14 / 12 = ~1.1667
    --scale: calc(14 / 12);
  }

  &&:focus-visible {
    outline: 2px solid ${colors.white};
    outline-offset: 2px;
  }
`;

export function SwitchTrigger(props: SwitchTriggerProps) {
  const { checked, disabled, onCheckedChange } = useSwitchContext();
  const handleClick = React.useCallback(() => {
    if (onCheckedChange) {
      onCheckedChange(!checked);
    }
  }, [checked, onCheckedChange]);

  return <StyledSwitchTrigger onClick={handleClick} disabled={disabled} {...props} />;
}
