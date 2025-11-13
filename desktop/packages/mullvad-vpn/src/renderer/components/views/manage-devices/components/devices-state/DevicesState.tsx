import { Spinner } from '../../../../../lib/components';
import { View } from '../../../../../lib/components/view';
import { useManageDevicesContext } from '../../ManageDevicesContext';
import { DevicesEmptyState } from '../devices-empty-state';

export function DevicesState() {
  const { isLoading } = useManageDevicesContext();
  return (
    <View.Container
      flexDirection="column"
      gap="tiny"
      alignItems="center"
      padding={{ bottom: 'tiny' }}>
      {isLoading ? <Spinner size="big" /> : <DevicesEmptyState />}
    </View.Container>
  );
}
