import React from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { DisableRecentsDialog } from '../../../../../features/locations/components';
import { useRecents } from '../../../../../features/locations/hooks';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { Menu, type MenuProps } from '../../../../../lib/components/menu';
import { useHistory } from '../../../../../lib/history';

export type HeaderMenuProps = MenuProps;

export function HeaderMenu({ onOpenChange, ...props }: HeaderMenuProps) {
  const history = useHistory();
  const { hasRecents, setEnabledRecents } = useRecents();
  const { multihop, setMultihop } = useMultihop();
  const navigateToFilter = React.useCallback(() => history.push(RoutePath.filter), [history]);

  const [disableRecentsDialogOpen, setDisableRecentsDialogOpen] = React.useState(false);

  const openDisableRecentsDialog = React.useCallback(() => {
    setDisableRecentsDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  const enableRecents = React.useCallback(async () => {
    await setEnabledRecents(true);
    onOpenChange?.(false);
  }, [onOpenChange, setEnabledRecents]);

  const toggleMultihop = React.useCallback(async () => {
    await setMultihop({ enabled: !multihop });
    onOpenChange?.(false);
  }, [multihop, onOpenChange, setMultihop]);

  return (
    <>
      <Menu onOpenChange={onOpenChange} {...props}>
        <Menu.Popup>
          <Menu.Option>
            <Menu.Option.Trigger onClick={navigateToFilter}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="filter" />
                <Menu.Option.Item.Label>{messages.gettext('Filters')}</Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Option>
            <Menu.Option.Trigger onClick={toggleMultihop}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="location-add" />
                <Menu.Option.Item.Label>
                  {multihop
                    ? // This line is here to prevent the following one to be moved up here by prettier
                      // TRANSLATORS: Used in button to disable multihop.
                      messages.pgettext('select-location-view', 'Disable multihop')
                    : // This line is here to prevent the following one to be moved up here by prettier
                      // TRANSLATORS: Used in button to enable multihop.
                      messages.pgettext('select-location-view', 'Enable multihop')}
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Option>
            <Menu.Option.Trigger onClick={hasRecents ? openDisableRecentsDialog : enableRecents}>
              <Menu.Option.Item>
                <Menu.Option.Item.Icon icon="history-remove" />
                <Menu.Option.Item.Label>
                  {hasRecents
                    ? // TRANSLATORS: Used in button to disable showing list of recent locations.
                      messages.pgettext('select-location-view', 'Disable recents')
                    : // TRANSLATORS: Used in button to enable showing list of recent locations.
                      messages.pgettext('select-location-view', 'Enable recents')}
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
        </Menu.Popup>
      </Menu>
      <DisableRecentsDialog
        open={disableRecentsDialogOpen}
        onOpenChange={setDisableRecentsDialogOpen}
      />
    </>
  );
}
