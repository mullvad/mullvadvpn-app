import { ListItem, ListItemProps } from '../../lib/components/list-item';
import { Switch, SwitchProps } from '../../lib/components/switch';
import { ToggleListItemLabel, ToggleListItemSwitch } from './components';

export type ToggleListItemProps = ListItemProps & {
  footer?: string;
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
};

function ToggleListItem({
  children,
  footer,
  checked,
  onCheckedChange,
  disabled,
  ...props
}: ToggleListItemProps) {
  return (
    <ListItem disabled={disabled} {...props}>
      <ListItem.Item>
        <ListItem.Content>
          <Switch checked={checked} onCheckedChange={onCheckedChange} disabled={disabled}>
            {children}
          </Switch>
        </ListItem.Content>
      </ListItem.Item>
      {footer && (
        <ListItem.Footer>
          <ListItem.Text>{footer}</ListItem.Text>
        </ListItem.Footer>
      )}
    </ListItem>
  );
}
const ToggleListItemNamespace = Object.assign(ToggleListItem, {
  Label: ToggleListItemLabel,
  Text: ListItem.Text,
  Group: ListItem.Group,
  Footer: ListItem.Footer,
  Switch: ToggleListItemSwitch,
});

export { ToggleListItemNamespace as ToggleListItem };
