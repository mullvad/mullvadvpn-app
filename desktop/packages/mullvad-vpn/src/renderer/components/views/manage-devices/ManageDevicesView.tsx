import { messages } from '../../../../shared/gettext';
import { Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { DeviceList } from '../../device-list';
import { BackAction } from '../../KeyboardNavigation';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { DevicesState } from './components';
import { useQueryDevices } from './hooks';
import { ManageDevicesProvider } from './ManageDevicesContext';

export function ManageDevicesView() {
  const { pop } = useHistory();
  const { devices, isFetching, isLoading, refetch } = useQueryDevices();
  const showDeviceList = devices && devices.length > 0;

  return (
    <ManageDevicesProvider isLoading={isLoading} isFetching={isFetching} refetchDevices={refetch}>
      <BackAction action={pop}>
        <View backgroundColor="darkBlue">
          <NavigationContainer>
            <AppNavigationHeader
              title={
                // TRANSLATORS: Title label in navigation bar for the manage devices view.
                messages.pgettext('device-management', 'Manage devices')
              }
            />
            <NavigationScrollbars>
              <FlexColumn gap="medium">
                <View.Container>
                  <FlexColumn gap="small">
                    <Text variant="titleBig">
                      {
                        // TRANSLATORS: Title text in the manage devices view
                        messages.pgettext('device-management', 'Manage devices')
                      }
                    </Text>
                    <Text variant="labelTiny" color="whiteAlpha60">
                      {
                        // TRANSLATORS: Subtitle text in the manage devices view, explaining
                        // TRANSLATORS: devices and what they can do in the manage devices view.
                        messages.pgettext(
                          'device-management',
                          'View and manage all your logged in devices. You can have up to 5 devices on one account at a time. Each device gets a name when logged in to help you tell them apart easily.',
                        )
                      }
                    </Text>
                  </FlexColumn>
                </View.Container>
                {showDeviceList ? <DeviceList devices={devices} /> : <DevicesState />}
              </FlexColumn>
            </NavigationScrollbars>
          </NavigationContainer>
        </View>
      </BackAction>
    </ManageDevicesProvider>
  );
}
