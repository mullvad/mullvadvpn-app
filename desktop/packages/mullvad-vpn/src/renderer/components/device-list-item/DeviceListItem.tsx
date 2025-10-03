import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { capitalizeEveryWord } from '../../../shared/string-helpers';
import { IconButton, Text } from '../../lib/components';
import { FlexColumn } from '../../lib/components/flex-column';
import { ListItem, ListItemProps } from '../../lib/components/list-item';
import { spacings } from '../../lib/foundations';
import { useBoolean } from '../../lib/utility-hooks';
import { DeviceListItemProvider, useDeviceListItemContext } from './';
import { ConfirmDialog } from './components';
import { useIsCurrentDevice } from './components/confirm-dialog/hooks';
import { ErrorDialog } from './ErrorDialog';

export type SettingsToggleListItemProps = {
  device: IDevice;
} & Omit<ListItemProps, 'children'>;

const StyledCurrentListItem = styled(ListItem)`
  margin-bottom: ${spacings.medium};
`;

function DeviceListItemInner({ ...props }: Omit<SettingsToggleListItemProps, 'device'>) {
  const { device, deleting, showConfirmDialog, confirmDialogVisible, error } =
    useDeviceListItemContext();
  const isCurrentDevice = useIsCurrentDevice();
  const deviceName = capitalizeEveryWord(device.name);
  const createdDate = device.created.toISOString().split('T')[0];

  const ListItemComponent = isCurrentDevice ? StyledCurrentListItem : ListItem;

  return (
    <>
      <ListItemComponent data-testid={'device-list-item'} disabled={deleting} {...props}>
        <ListItem.Item>
          <ListItem.Content>
            <FlexColumn>
              <ListItem.Label>{deviceName}</ListItem.Label>
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
            <ListItem.Group>
              {isCurrentDevice && (
                <Text variant="labelTiny" color="whiteAlpha60">
                  {
                    // TRANSLATORS: Label indicating that this device is the current device.
                    messages.pgettext('device-management', 'Current device')
                  }
                </Text>
              )}
              {!isCurrentDevice && (
                <IconButton
                  variant="secondary"
                  onClick={showConfirmDialog}
                  disabled={deleting}
                  aria-label={sprintf(
                    // TRANSLATORS: Button action description provided to accessibility tools such as screen
                    // TRANSLATORS: readers.
                    // TRANSLATORS: Available placeholders:
                    // TRANSLATORS: %(deviceName)s - The device name to remove.
                    messages.pgettext('accessibility', 'Remove device named %(deviceName)s'),
                    { deviceName: device.name },
                  )}>
                  <IconButton.Icon icon="cross-circle" />
                </IconButton>
              )}
            </ListItem.Group>
          </ListItem.Content>
        </ListItem.Item>
      </ListItemComponent>
      <ConfirmDialog isOpen={confirmDialogVisible} />
      <ErrorDialog isOpen={error} />
    </>
  );
}

export function DeviceListItem({ device, ...props }: SettingsToggleListItemProps) {
  const [confirmDialogVisible, showConfirmDialog, hideConfirmDialog] = useBoolean(false);
  const [error, setError, resetError] = useBoolean(false);
  const [deleting, setDeleting, resetDeleting] = useBoolean(false);

  return (
    <DeviceListItemProvider
      device={device}
      deleting={deleting}
      setDeleting={setDeleting}
      resetDeleting={resetDeleting}
      confirmDialogVisible={confirmDialogVisible}
      showConfirmDialog={showConfirmDialog}
      hideConfirmDialog={hideConfirmDialog}
      error={error}
      resetError={resetError}
      setError={setError}>
      <DeviceListItemInner {...props} />
    </DeviceListItemProvider>
  );
}
