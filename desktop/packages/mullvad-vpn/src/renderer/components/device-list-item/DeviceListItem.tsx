import { sprintf } from 'sprintf-js';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { FlexColumn } from '../../lib/components/flex-column';
import { ListItem, ListItemProps } from '../../lib/components/list-item';
import { formatDeviceName } from '../../lib/utils';
import { DeviceListItemProvider, useDeviceListItemContext } from './';
import { ConfirmDialog, ErrorDialog, RemoveButton } from './components';
import { useFormattedDate, useIsCurrentDevice } from './hooks';

export type SettingsToggleListItemProps = {
  device: IDevice;
} & Omit<ListItemProps, 'children'>;

function DeviceListItemInner({ ...props }: Omit<SettingsToggleListItemProps, 'device'>) {
  const { device, deleting, confirmDialogVisible, error } = useDeviceListItemContext();
  const createdDate = useFormattedDate(device.created);
  const isCurrentDevice = useIsCurrentDevice();

  return (
    <>
      <ListItem disabled={deleting} {...props}>
        <ListItem.Item>
          <FlexColumn>
            <ListItem.Label>{formatDeviceName(device.name)}</ListItem.Label>
            <ListItem.Text variant="footnoteMini">
              {sprintf(
                // TRANSLATORS: Label informing the user when a device was created.
                // TRANSLATORS: Available placeholders:
                // TRANSLATORS: %(createdDate)s - The creation date of the device.
                messages.pgettext('device-management', 'Created: %(createdDate)s'),
                {
                  createdDate,
                },
              )}
            </ListItem.Text>
          </FlexColumn>
          <ListItem.ActionGroup>
            {isCurrentDevice ? (
              <ListItem.Text variant="labelTiny" color="whiteAlpha60">
                {
                  // TRANSLATORS: Label indicating that this device is the current device.
                  messages.pgettext('device-management', 'Current device')
                }
              </ListItem.Text>
            ) : (
              <RemoveButton />
            )}
          </ListItem.ActionGroup>
        </ListItem.Item>
      </ListItem>
      <ConfirmDialog isOpen={confirmDialogVisible} />
      <ErrorDialog isOpen={error} />
    </>
  );
}

export function DeviceListItem({ device, ...props }: SettingsToggleListItemProps) {
  return (
    <DeviceListItemProvider device={device}>
      <DeviceListItemInner {...props} />
    </DeviceListItemProvider>
  );
}
