import { Switch } from '../../../../lib/components/switch';
import { SwitchLabelProps } from '../../../../lib/components/switch/components/switch-label';

export type SettingsToggleListItemLabelProps = SwitchLabelProps;

export function SettingsToggleListItemLabel(props: SettingsToggleListItemLabelProps) {
  return <Switch.Label variant="titleMedium" {...props} />;
}
