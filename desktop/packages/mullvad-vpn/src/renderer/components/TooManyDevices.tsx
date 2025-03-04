import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { IDevice } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { capitalizeEveryWord } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { Button, Flex, IconButton, Spinner } from '../lib/components';
import { Colors, Spacings } from '../lib/foundations';
import { transitions, useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import { IconBadge, IconBadgeProps } from '../lib/icon-badge';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utility-hooks';
import { useSelector } from '../redux/store';
import { AppMainHeader } from './app-main-header';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { bigText, measurements, normalText, tinyText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Footer, Layout, SettingsContainer } from './Layout';
import List from './List';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

const StyledContainer = styled(SettingsContainer)({
  minHeight: '100%',
});

const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: 'auto',
});

const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  margin: `0 ${measurements.horizontalViewMargin} 8px`,
  color: Colors.white,
});

const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '12px',
  fontWeight: 600,
  lineHeight: '20px',
  color: Colors.white,
  margin: `0 ${measurements.horizontalViewMargin} 18px`,
});

const StyledSpacer = styled.div({
  flex: '1',
});

const StyledDeviceInfo = styled(Cell.Label)({
  display: 'flex',
  flexDirection: 'column',
  marginTop: '9px',
  marginBottom: '9px',
});

const StyledDeviceName = styled.span(normalText, {
  fontWeight: 'normal',
  lineHeight: '20px',
  textTransform: 'capitalize',
});

const StyledDeviceDate = styled.span(tinyText, {
  fontSize: '10px',
  lineHeight: '10px',
  color: Colors.white60,
});

export default function TooManyDevices() {
  const { reset } = useHistory();
  const { removeDevice, login, cancelLogin } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const devices = useSelector((state) => state.account.devices);
  const loginState = useSelector((state) => state.account.status);

  const onRemoveDevice = useCallback(
    async (deviceId: string) => {
      await removeDevice({ accountNumber, deviceId });
    },
    [removeDevice, accountNumber],
  );

  const continueLogin = useCallback(() => {
    void login(accountNumber);
    reset(RoutePath.login, { transition: transitions.pop });
  }, [reset, login, accountNumber]);
  const cancel = useCallback(() => {
    cancelLogin();
    reset(RoutePath.login, { transition: transitions.pop });
  }, [reset, cancelLogin]);

  const imageSource = getIconSource(devices);
  const title = getTitle(devices);
  const subtitle = getSubtitle(devices);

  const continueButtonDisabled = devices.length === 5 || loginState.type !== 'too many devices';

  return (
    <ModalContainer>
      <Layout>
        <AppMainHeader>
          <AppMainHeader.SettingsButton />
        </AppMainHeader>
        <StyledCustomScrollbars fillContainer>
          <StyledContainer>
            <StyledBody>
              <Flex
                $justifyContent="center"
                $margin={{ top: Spacings.large, bottom: Spacings.medium }}>
                <IconBadge key={imageSource} state={imageSource} />
              </Flex>
              {devices !== undefined && (
                <>
                  <StyledTitle data-testid="title">{title}</StyledTitle>
                  <StyledLabel>{subtitle}</StyledLabel>
                  <DeviceList devices={devices} onRemoveDevice={onRemoveDevice} />
                </>
              )}
            </StyledBody>

            {devices !== undefined && (
              <Footer>
                <AppButton.ButtonGroup>
                  <Button
                    variant="success"
                    onClick={continueLogin}
                    disabled={continueButtonDisabled}>
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
                </AppButton.ButtonGroup>
              </Footer>
            )}
          </StyledContainer>
        </StyledCustomScrollbars>
      </Layout>
    </ModalContainer>
  );
}

interface IDeviceListProps {
  devices: Array<IDevice>;
  onRemoveDevice: (deviceId: string) => Promise<void>;
}

function DeviceList(props: IDeviceListProps) {
  return (
    <StyledSpacer>
      <List items={props.devices} getKey={getDeviceKey}>
        {(device) => <Device device={device} onRemove={props.onRemoveDevice} />}
      </List>
    </StyledSpacer>
  );
}

const getDeviceKey = (device: IDevice): string => device.id;

interface IDeviceProps {
  device: IDevice;
  onRemove: (deviceId: string) => Promise<void>;
}

function Device(props: IDeviceProps) {
  const { onRemove: propsOnRemove } = props;

  const { fetchDevices } = useAppContext();
  const accountNumber = useSelector((state) => state.account.accountNumber)!;
  const [confirmationVisible, showConfirmation, hideConfirmation] = useBoolean(false);
  const [deleting, setDeleting, unsetDeleting] = useBoolean(false);
  const [error, setError, resetError] = useBoolean(false);

  const handleError = useCallback(
    async (error: Error) => {
      log.error(`Failed to remove device: ${error.message}`);

      let devices: Array<IDevice> | undefined = undefined;
      try {
        devices = await fetchDevices(accountNumber);
      } catch {
        /* no-op */
      }

      if (devices === undefined || devices.find((device) => device.id === props.device.id)) {
        hideConfirmation();
        unsetDeleting();
        setError();
      }
    },
    [fetchDevices, accountNumber, props.device.id, hideConfirmation, unsetDeleting, setError],
  );

  const onRemove = useCallback(async () => {
    setDeleting();
    hideConfirmation();
    try {
      await propsOnRemove(props.device.id);
    } catch (e) {
      await handleError(e as Error);
    }
  }, [propsOnRemove, props.device.id, hideConfirmation, setDeleting, handleError]);

  const capitalizedDeviceName = capitalizeEveryWord(props.device.name);
  const createdDate = props.device.created.toISOString().split('T')[0];

  return (
    <>
      <Cell.Container>
        <StyledDeviceInfo>
          <StyledDeviceName aria-hidden>{props.device.name}</StyledDeviceName>
          <StyledDeviceDate>
            {sprintf(
              // TRANSLATORS: Label informing the user when a device was created.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(createdDate)s - The creation date of the device.
              messages.pgettext('device-management', 'Created: %(createdDate)s'),
              {
                createdDate,
              },
            )}
          </StyledDeviceDate>
        </StyledDeviceInfo>
        {deleting ? (
          <Spinner />
        ) : (
          <IconButton
            variant="secondary"
            onClick={showConfirmation}
            aria-label={sprintf(
              // TRANSLATORS: Button action description provided to accessibility tools such as screen
              // TRANSLATORS: readers.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(deviceName)s - The device name to remove.
              messages.pgettext('accessibility', 'Remove device named %(deviceName)s'),
              { deviceName: props.device.name },
            )}>
            <IconButton.Icon icon="cross-circle" />
          </IconButton>
        )}
      </Cell.Container>
      <ModalAlert
        isOpen={confirmationVisible}
        type={ModalAlertType.warning}
        iconColor={Colors.red}
        buttons={[
          <AppButton.RedButton key="remove" onClick={onRemove} disabled={deleting}>
            {
              // TRANSLATORS: Confirmation button when logging out other device.
              messages.pgettext('device-management', 'Yes, log out device')
            }
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={hideConfirmation} disabled={deleting}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ]}
        close={hideConfirmation}>
        <ModalMessage>
          {formatHtml(
            sprintf(
              // TRANSLATORS: Text displayed above button which logs out another device.
              // TRANSLATORS: The text enclosed in "<b></b>" will appear bold.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(deviceName)s - The name of the device to log out.
              messages.pgettext(
                'device-management',
                'Are you sure you want to log <b>%(deviceName)s</b> out?',
              ),
              { deviceName: capitalizedDeviceName },
            ),
          )}
        </ModalMessage>
      </ModalAlert>
      <ModalAlert
        isOpen={error}
        type={ModalAlertType.warning}
        iconColor={Colors.red}
        buttons={[
          <AppButton.BlueButton key="close" onClick={resetError}>
            {messages.gettext('Close')}
          </AppButton.BlueButton>,
        ]}
        close={resetError}
        message={messages.pgettext('device-management', 'Failed to remove device')}
      />
    </>
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
