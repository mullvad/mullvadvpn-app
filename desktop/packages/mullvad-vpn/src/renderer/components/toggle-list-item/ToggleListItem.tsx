import { Switch, SwitchProps } from '../../lib/components/switch';
import { SettingsListItem, SettingsListItemProps } from '../settings-list-item';
import { ToggleListItemLabel, ToggleListItemSwitch } from './components';

export type ToggleListItemProps = {
  footer?: string;
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
} & SettingsListItemProps;

function ToggleListItem({
  ref,
  children,
  footer,
  checked,
  onCheckedChange,
  disabled,
  ...props
}: ToggleListItemProps) {
  return (
    <SettingsListItem disabled={disabled} {...props}>
      <SettingsListItem.Item>
        <SettingsListItem.Content>
          <Switch checked={checked} onCheckedChange={onCheckedChange} disabled={disabled}>
            {children}
          </Switch>
        </SettingsListItem.Content>
      </SettingsListItem.Item>
      {footer && (
        <SettingsListItem.Footer>
          <SettingsListItem.Text>{footer}</SettingsListItem.Text>
        </SettingsListItem.Footer>
      )}
    </SettingsListItem>
  );
}
const ToggleListItemNamespace = Object.assign(ToggleListItem, {
  Label: ToggleListItemLabel,
  Text: SettingsListItem.Text,
  Group: SettingsListItem.Group,
  Footer: SettingsListItem.Footer,
  Switch: ToggleListItemSwitch,
});

export { ToggleListItemNamespace as ToggleListItem };
