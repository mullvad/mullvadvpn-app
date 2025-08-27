import { Switch } from '../../../../lib/components/switch';
import { SwitchLabelProps } from '../../../../lib/components/switch/components/switch-label';

export type ToggleListItemLabelProps = SwitchLabelProps;

export function ToggleListItemLabel(props: ToggleListItemLabelProps) {
  return <Switch.Label variant="titleMedium" {...props} />;
}
