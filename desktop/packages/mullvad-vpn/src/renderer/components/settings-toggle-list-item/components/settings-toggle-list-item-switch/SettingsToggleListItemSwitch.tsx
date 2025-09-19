import styled from 'styled-components';

import { Switch } from '../../../../lib/components/switch';
import { SwitchTriggerProps } from '../../../../lib/components/switch/components';
import { spacings } from '../../../../lib/foundations';
import { useSettingsToggleListItemContext } from '../../SettingsToggleListItemContext';

export type SettingsToggleListItemSwitchProps = SwitchTriggerProps;

export const StyledSettingsToggleListItemSwitch = styled(Switch.Trigger)`
  margin-left: ${spacings.small};
`;

export function SettingsToggleListItemSwitch(props: SettingsToggleListItemSwitchProps) {
  const { descriptionId } = useSettingsToggleListItemContext();
  return (
    <Switch.Trigger aria-describedby={descriptionId} {...props}>
      <Switch.Thumb />
    </Switch.Trigger>
  );
}
