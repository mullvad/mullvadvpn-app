import React, { useCallback } from 'react';

import { messages } from '../../../../../../shared/gettext';
import { RoutePath } from '../../../../../../shared/routes';
import { DisableRecentsDialog } from '../../../../../features/locations/components';
import { useRecents } from '../../../../../features/locations/hooks';
import { LocationType } from '../../../../../features/locations/types';
import { useMultihop } from '../../../../../features/multihop/hooks';
import { Menu, type MenuProps } from '../../../../../lib/components/menu';
import { useHistory } from '../../../../../lib/history';
import { useSelectLocationViewContext } from '../../SelectLocationViewContext';

export type HeaderMenuProps = MenuProps;

export function HeaderMenu({ onOpenChange, ...props }: HeaderMenuProps) {
  const history = useHistory();
  const { hasRecents, setEnabledRecents } = useRecents();
  const { multihop, setMultihop } = useMultihop();
  const { setLocationType } = useSelectLocationViewContext();
  const navigateToFilter = React.useCallback(() => history.push(RoutePath.filter), [history]);

  const [disableRecentsDialogOpen, setDisableRecentsDialogOpen] = React.useState(false);

  const openDisableRecentsDialog = React.useCallback(() => {
    setDisableRecentsDialogOpen(true);
    onOpenChange?.(false);
  }, [onOpenChange]);

  const enableRecents = React.useCallback(async () => {
    await setEnabledRecents(true);
    onOpenChange?.(false);
    setLocationType(LocationType.exit);
  }, [onOpenChange, setEnabledRecents, setLocationType]);

  const handleMultihopAlways = useCallback(async () => {
    await setMultihop({ multihop: 'always' });
    onOpenChange?.(false);
    setLocationType(LocationType.entry);
  }, [onOpenChange, setLocationType, setMultihop]);

  const handleMultihopNever = useCallback(async () => {
    await setMultihop({ multihop: 'never' });
    setLocationType(LocationType.exit);
    onOpenChange?.(false);
  }, [onOpenChange, setLocationType, setMultihop]);

  const handleMultihopWhenNeeded = useCallback(async () => {
    await setMultihop({ multihop: 'when-needed' });
    setLocationType(LocationType.exit);
    onOpenChange?.(false);
  }, [onOpenChange, setLocationType, setMultihop]);

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
          <Menu.Title>
            {
              // TRANSLATORS: Title for a menu with items for the Multihop mode setting options.
              messages.pgettext('select-location-view', 'Multihop mode')
            }
          </Menu.Title>
          <Menu.Option>
            <Menu.Option.Trigger onClick={handleMultihopWhenNeeded}>
              <Menu.Option.Item>
                {multihop === 'when-needed' ? (
                  <Menu.Option.Item.Icon icon="checkmark" />
                ) : (
                  <Menu.Option.Item.Icon icon="placeholder" />
                )}
                <Menu.Option.Item.Label>
                  {
                    // TRANSLATORS: Label for a menu option to change the Multihop mode setting
                    // TRANSLATORS: option to "When needed".
                    messages.pgettext('select-location-view', 'When needed')
                  }
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Option>
            <Menu.Option.Trigger onClick={handleMultihopAlways}>
              <Menu.Option.Item>
                {multihop === 'always' ? (
                  <Menu.Option.Item.Icon icon="checkmark" />
                ) : (
                  <Menu.Option.Item.Icon icon="placeholder" />
                )}
                <Menu.Option.Item.Label>
                  {
                    // TRANSLATORS: Label for a menu option to change the Multihop mode setting
                    // TRANSLATORS: option to "Always".
                    messages.pgettext('select-location-view', 'Always')
                  }
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Option>
            <Menu.Option.Trigger onClick={handleMultihopNever}>
              <Menu.Option.Item>
                {multihop === 'never' ? (
                  <Menu.Option.Item.Icon icon="checkmark" />
                ) : (
                  <Menu.Option.Item.Icon icon="placeholder" />
                )}
                <Menu.Option.Item.Label>
                  {
                    // TRANSLATORS: Label for a menu option to change the Multihop mode setting
                    // TRANSLATORS: option to "Never".
                    messages.pgettext('select-location-view', 'Never')
                  }
                </Menu.Option.Item.Label>
              </Menu.Option.Item>
            </Menu.Option.Trigger>
          </Menu.Option>
          <Menu.Divider />
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
