import { useCallback } from 'react';
import styled from 'styled-components';

import { IDevice } from '../../../../shared/daemon-rpc-types';
import { messages } from '../../../../shared/gettext';
import { RoutePath } from '../../../../shared/routes';
import { useAppContext } from '../../../context';
import { Button, Flex, Text } from '../../../lib/components';
import { FlexColumn } from '../../../lib/components/flex-column';
import { View } from '../../../lib/components/view';
import { TransitionType, useHistory } from '../../../lib/history';
import { IconBadge, IconBadgeProps } from '../../../lib/icon-badge';
import { useSelector } from '../../../redux/store';
import { AppMainHeader } from '../../app-main-header';
import CustomScrollbars from '../../CustomScrollbars';
import { DeviceList } from '../../device-list';

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

export function TooManyDevicesView() {
  const { reset } = useHistory();
  const { login, cancelLogin } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSelector((state) => state.account.devices);
  const loginState = useSelector((state) => state.account.status);

  const continueLogin = useCallback(() => {
    void login(accountNumber);
    reset(RoutePath.login, { transition: TransitionType.pop });
  }, [reset, login, accountNumber]);

  const cancel = useCallback(() => {
    cancelLogin();
    reset(RoutePath.login, { transition: TransitionType.pop });
  }, [reset, cancelLogin]);

  const imageSource = getIconSource(devices);
  const title = getTitle(devices);
  const subtitle = getSubtitle(devices);

  const continueButtonDisabled = devices.length === 5 || loginState.type !== 'too many devices';

  return (
    <View backgroundColor="darkBlue">
      <AppMainHeader>
        <AppMainHeader.SettingsButton />
      </AppMainHeader>
      <StyledCustomScrollbars fillContainer>
        <FlexColumn gap="large">
          <View.Container flexDirection="column" size="4">
            <Flex justifyContent="center" margin={{ top: 'large' }}>
              <IconBadge key={imageSource} state={imageSource} />
            </Flex>
          </View.Container>
          {devices !== undefined && (
            <>
              <View.Container flexDirection="column" size="4" gap="small">
                <Text variant="titleLarge" data-testid="title">
                  {title}
                </Text>
                <Text variant="labelTiny">{subtitle}</Text>
              </View.Container>
              <DeviceList devices={devices} />
            </>
          )}

          {devices !== undefined && (
            <View.Container
              flexDirection="column"
              size="4"
              gap="medium"
              padding={{ bottom: 'large' }}>
              <Button variant="success" onClick={continueLogin} disabled={continueButtonDisabled}>
                <Button.Text>
                  {
                    // TRANSLATORS: Button for continuing login process.
                    messages.pgettext('device-management', 'Continue with login')
                  }
                </Button.Text>
              </Button>
              <Button onClick={cancel}>
                <Button.Text>{messages.gettext('Back')}</Button.Text>
              </Button>
            </View.Container>
          )}
        </FlexColumn>
      </StyledCustomScrollbars>
    </View>
  );
}

function getIconSource(devices: Array<IDevice>): IconBadgeProps['state'] {
  if (devices.length === 5) {
    return 'negative';
  } else {
    return 'positive';
  }
}

function getTitle(devices?: Array<IDevice>): string | undefined {
  if (devices) {
    if (devices.length === 5) {
      // TRANSLATORS: Page title informing user that the login failed due to too many registered
      // TRANSLATORS: devices on account.
      return messages.pgettext('device-management', 'Too many devices');
    } else {
      // TRANSLATORS: Page title informing user that enough devices has been removed to continue
      // TRANSLATORS: login process.
      return messages.pgettext('device-management', 'Super!');
    }
  } else {
    return undefined;
  }
}

function getSubtitle(devices?: Array<IDevice>): string | undefined {
  if (devices) {
    if (devices.length === 5) {
      return messages.pgettext(
        'device-management',
        'Please log out of at least one by removing it from the list below. You can find the corresponding device name under the device’s Account settings.',
      );
    } else {
      return messages.pgettext(
        'device-management',
        'You can now continue logging in on this device.',
      );
    }
  } else {
    return undefined;
  }
}
