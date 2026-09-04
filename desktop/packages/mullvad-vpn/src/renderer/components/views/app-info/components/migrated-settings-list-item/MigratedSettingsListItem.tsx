import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { useSettingsMigrations } from '../../../../../features/migration/hooks';
import { Flex, Icon } from '../../../../../lib/components';
import { Dot } from '../../../../../lib/components/dot';
import { ListItem, ListItemProps } from '../../../../../lib/components/list-item';
import { useHistory } from '../../../../../lib/history';

export type MigratedSettingsListItemProps = Omit<ListItemProps, 'children'>;

export function MigratedSettingsListItem(props: MigratedSettingsListItemProps) {
  const { push } = useHistory();

  const handleClick = React.useCallback(() => {
    push(RoutePath.migratedSettings);
  }, [push]);

  const { displayedSettingsMigrations, hasSettingsMigrations } = useSettingsMigrations();

  return (
    <ListItem {...props}>
      <ListItem.Trigger onClick={handleClick}>
        <ListItem.Item>
          <Flex flexDirection="column">
            <ListItem.Item.Label>
              {
                // TRANSLATORS: Label for migrated settings list item.
                messages.pgettext('app-info-view', 'Migrated settings')
              }
            </ListItem.Item.Label>
          </Flex>
          <ListItem.Item.ActionGroup>
            {!displayedSettingsMigrations && hasSettingsMigrations && (
              <Dot variant="warning" size="small" />
            )}
            <Icon icon="chevron-right" />
          </ListItem.Item.ActionGroup>
        </ListItem.Item>
      </ListItem.Trigger>
    </ListItem>
  );
}
