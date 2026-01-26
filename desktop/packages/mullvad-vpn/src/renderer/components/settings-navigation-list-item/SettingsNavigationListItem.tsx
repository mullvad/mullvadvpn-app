import React from 'react';

import { RoutePath } from '../../../shared/routes';
import { useHistory } from '../../lib/history';
import { SettingsListItem, SettingsListItemProps } from '../settings-list-item';

export type SettingsNavigationListItemProps = {
  to: RoutePath;
} & SettingsListItemProps;

function SettingsNavigationListItem({ to, children, ...props }: SettingsNavigationListItemProps) {
  const history = useHistory();
  const navigate = React.useCallback(() => history.push(to), [history, to]);

  return (
    <SettingsListItem {...props}>
      <SettingsListItem.Trigger onClick={navigate}>
        <SettingsListItem.Item>{children}</SettingsListItem.Item>
      </SettingsListItem.Trigger>
    </SettingsListItem>
  );
}

const SettingsNavigationListItemNamespace = Object.assign(SettingsNavigationListItem, {
  Label: SettingsListItem.Label,
  Group: SettingsListItem.Group,
  ActionGroup: SettingsListItem.ActionGroup,
  Text: SettingsListItem.Text,
  Footer: SettingsListItem.Footer,
  Icon: SettingsListItem.Icon,
});

export { SettingsNavigationListItemNamespace as SettingsNavigationListItem };
