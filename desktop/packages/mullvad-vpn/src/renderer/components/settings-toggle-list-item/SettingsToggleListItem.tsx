import React from 'react';

import { Switch, SwitchProps } from '../../lib/components/switch';
import { SettingsListItem, SettingsListItemProps } from '../settings-list-item';
import {
  SettingsToggleListItemGroup,
  SettingsToggleListItemLabel,
  SettingsToggleListItemSwitch,
} from './components';
import { SettingsToggleListItemProvider } from './SettingsToggleListItemContext';

export type SettingsToggleListItemProps = {
  description?: string;
  checked?: SwitchProps['checked'];
  onCheckedChange?: SwitchProps['onCheckedChange'];
} & SettingsListItemProps;

function SettingsToggleListItem({
  ref,
  children,
  description,
  checked,
  onCheckedChange,
  disabled,
  ...props
}: SettingsToggleListItemProps) {
  const descriptionId = React.useId();
  const labelId = React.useId();
  return (
    <SettingsToggleListItemProvider descriptionId={descriptionId}>
      <SettingsListItem labelId={labelId} disabled={disabled} {...props}>
        <SettingsListItem.Item>
          <SettingsListItem.Content>
            <Switch
              labelId={labelId}
              checked={checked}
              onCheckedChange={onCheckedChange}
              disabled={disabled}
              aria-describedby={description ? descriptionId : undefined}>
              {children}
            </Switch>
          </SettingsListItem.Content>
        </SettingsListItem.Item>
        {description && (
          <SettingsListItem.Footer>
            <SettingsListItem.Text id={descriptionId}>{description}</SettingsListItem.Text>
          </SettingsListItem.Footer>
        )}
      </SettingsListItem>
    </SettingsToggleListItemProvider>
  );
}
const SettingsToggleListItemNamespace = Object.assign(SettingsToggleListItem, {
  Label: SettingsToggleListItemLabel,
  Text: SettingsListItem.Text,
  Group: SettingsToggleListItemGroup,
  Footer: SettingsListItem.Footer,
  Switch: SettingsToggleListItemSwitch,
});

export { SettingsToggleListItemNamespace as SettingsToggleListItem };
