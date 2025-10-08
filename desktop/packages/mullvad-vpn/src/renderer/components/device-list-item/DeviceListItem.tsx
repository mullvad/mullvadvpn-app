import { sprintf } from 'sprintf-js';
import styled, { css } from 'styled-components';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { messages } from '../../../shared/gettext';
import { Text } from '../../lib/components';
import { FlexColumn } from '../../lib/components/flex-column';
import { ListItem, ListItemProps } from '../../lib/components/list-item';
import { spacings } from '../../lib/foundations';
import { useFormattedDeviceName } from '../../lib/hooks';
import { useBoolean } from '../../lib/utility-hooks';
import { DeviceListItemProvider, useDeviceListItemContext } from './';
import { ConfirmDialog, ErrorDialog, RemoveButton } from './components';
import { useFormattedDate, useIsCurrentDevice } from './hooks';

export type SettingsToggleListItemProps = {
  device: IDevice;
} & Omit<ListItemProps, 'children'>;

const StyledListItem = styled(ListItem)<{ $isCurrentDevice: boolean }>(
  ({ $isCurrentDevice }) => css`
    ${() => {
      if ($isCurrentDevice) {
        return css`
          margin-bottom: ${spacings.medium};
        `;
      }
      return null;
    }}
  `,
);

function DeviceListItemInner({ ...props }: Omit<SettingsToggleListItemProps, 'device'>) {
  const { device, deleting, confirmDialogVisible, error } = useDeviceListItemContext();
  const createdDate = useFormattedDate(device.created);
  const isCurrentDevice = useIsCurrentDevice();
  const deviceName = useFormattedDeviceName(device.name);

  return (
    <>
      <StyledListItem disabled={deleting} $isCurrentDevice={isCurrentDevice} {...props}>
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
              {isCurrentDevice ? (
                <Text variant="labelTiny" color="whiteAlpha60">
                  {
                    // TRANSLATORS: Label indicating that this device is the current device.
                    messages.pgettext('device-management', 'Current device')
                  }
                </Text>
              ) : (
                <RemoveButton />
              )}
            </ListItem.Group>
          </ListItem.Content>
        </ListItem.Item>
      </StyledListItem>
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
