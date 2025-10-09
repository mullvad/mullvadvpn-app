import { messages } from '../../../../../../shared/gettext';
import { EmptyState } from '../../../../../lib/components/empty-state';
import { useManageDevicesContext } from '../../ManageDevicesContext';

export function NoDevicesState() {
  const { isFetching, refetchDevices } = useManageDevicesContext();
  return (
    <EmptyState variant={isFetching ? 'loading' : 'error'} $alignSelf="stretch">
      <EmptyState.StatusIcon />
      <EmptyState.TextContainer>
        <EmptyState.Title>
          {
            // TRANSLATORS: Title text when devices could not be fetched.
            messages.pgettext('device-management', 'Failed to fetch list of devices')
          }
        </EmptyState.Title>
      </EmptyState.TextContainer>
      <EmptyState.Button onClick={refetchDevices}>
        <EmptyState.Button.Text>
          {
            // TRANSLATORS: Button text to retry fetching devices.
            messages.pgettext('device-management', 'Try again')
          }
        </EmptyState.Button.Text>
      </EmptyState.Button>
    </EmptyState>
  );
}
