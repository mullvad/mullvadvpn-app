import { Switch, SwitchProps } from '../../lib/components/switch';
import { SettingsListItem, SettingsListItemProps } from '../settings-list-item';
import { SettingsToggleListItemLabel, SettingsToggleListItemSwitch } from './components';

export type SettingsToggleListItemProps = {
  footer?: string;
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
} & SettingsListItemProps;

function SettingsToggleListItem({
  ref,
  children,
  footer,
  checked,
  onCheckedChange,
  disabled,
  ...props
}: SettingsToggleListItemProps) {
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
const SettingsToggleListItemNamespace = Object.assign(SettingsToggleListItem, {
  Label: SettingsToggleListItemLabel,
  Text: SettingsListItem.Text,
  Group: SettingsListItem.Group,
  Footer: SettingsListItem.Footer,
  Switch: SettingsToggleListItemSwitch,
});

export { SettingsToggleListItemNamespace as SettingsToggleListItem };
