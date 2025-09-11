import React from 'react';

import { RoutePath } from '../../shared/routes';
import { useHistory } from '../lib/history';
import { SettingsListItem, SettingsListItemProps } from './settings-list-item';

export type NavigationListItemProps = {
  to: RoutePath;
} & SettingsListItemProps;

function NavigationListItem({ to, children, ...props }: NavigationListItemProps) {
  const history = useHistory();
  const navigate = React.useCallback(() => history.push(to), [history, to]);

  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Trigger onClick={navigate}>
        <SettingsListItem.Item>
          <SettingsListItem.Content>{children}</SettingsListItem.Content>
        </SettingsListItem.Item>
      </SettingsListItem.Trigger>
    </SettingsListItem>
  );
}
const NavigationListItemNamespace = Object.assign(NavigationListItem, {
  Label: SettingsListItem.Label,
  Group: SettingsListItem.Group,
  Text: SettingsListItem.Text,
  Footer: SettingsListItem.Footer,
  Icon: SettingsListItem.Icon,
});

export { NavigationListItemNamespace as NavigationListItem };
