import { IDevice } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { Spinner, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { useHistory } from '../../../lib/history';
import { AppNavigationHeader } from '../../app-navigation-header';
import { DeviceListItem } from '../../device-list-item';
import { BackAction } from '../../KeyboardNavigation';
import List from '../../List';
import { NavigationContainer } from '../../NavigationContainer';
import { NavigationScrollbars } from '../../NavigationScrollbars';
import { useGetDevices } from './hooks';

const getDeviceKey = (device: IDevice): string => device.id;

export function ManageDevicesView() {
  const { pop } = useHistory();

  const { loading, devices } = useGetDevices();
  return (
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
            <FlexColumn $gap="medium">
              <View.Container>
                <FlexColumn $gap="small">
                  <Text variant="titleBig">
                    {messages.pgettext('device-management', 'Manage devices')}
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
              {loading && <Spinner />}
              {!loading && (
                <div>
                  <List items={devices} getKey={getDeviceKey} skipAddTransition>
                    {(device) => <DeviceListItem device={device} />}
                  </List>
                </div>
              )}
            </FlexColumn>
          </NavigationScrollbars>
        </NavigationContainer>
      </View>
    </BackAction>
  );
}
