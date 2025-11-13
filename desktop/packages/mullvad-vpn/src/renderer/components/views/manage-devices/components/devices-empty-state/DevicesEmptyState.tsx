import { messages } from '../../../../../../shared/gettext';
import { EmptyState } from '../../../../../lib/components/empty-state';
import { useManageDevicesContext } from '../../ManageDevicesContext';

export function DevicesEmptyState() {
  const { isFetching, refetchDevices } = useManageDevicesContext();
  return (
    <EmptyState variant={isFetching ? 'loading' : 'error'} alignSelf="stretch">
      <EmptyState.StatusIcon />
      <EmptyState.TextContainer>
        <EmptyState.Title>{messages.gettext('Failed to fetch list of devices')}</EmptyState.Title>
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
