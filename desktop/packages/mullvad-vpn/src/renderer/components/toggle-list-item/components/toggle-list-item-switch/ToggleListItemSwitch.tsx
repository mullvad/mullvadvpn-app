import styled from 'styled-components';

import { Switch } from '../../../../lib/components/switch';
import { SwitchTriggerProps } from '../../../../lib/components/switch/components';
import { spacings } from '../../../../lib/foundations';

export type ToggleListItemSwitchProps = SwitchTriggerProps;

export const StyledToggleListItemSwitch = styled(Switch.Trigger)`
  margin-left: ${spacings.small};
`;

export function ToggleListItemSwitch(props: ToggleListItemSwitchProps) {
  return (
    <StyledToggleListItemSwitch {...props}>
      <Switch.Thumb />
    </StyledToggleListItemSwitch>
  );
}
