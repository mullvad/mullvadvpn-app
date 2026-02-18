import React from 'react';
import styled from 'styled-components';

import { colors } from '../../../../foundations';
import { useCheckboxContext } from '../../CheckboxContext';

export type CheckboxTriggerProps = React.ComponentPropsWithRef<'button'>;

export const StyledCheckboxTrigger = styled.button`
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

export function CheckboxTrigger(props: CheckboxTriggerProps) {
  const { checked, disabled, onCheckedChange } = useCheckboxContext();
  const handleClick = React.useCallback(() => {
    if (onCheckedChange) {
      onCheckedChange(!checked);
    }
  }, [checked, onCheckedChange]);

  return <StyledCheckboxTrigger onClick={handleClick} disabled={disabled} {...props} />;
}
