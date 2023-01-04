import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { colors } from '../../config.json';
import { IDevice } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import log from '../../shared/logging';
import { capitalizeEveryWord } from '../../shared/string-helpers';
import { useAppContext } from '../context';
import { transitions, useHistory } from '../lib/history';
import { formatHtml } from '../lib/html-formatter';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { bigText, measurements } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Brand, HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Footer, Header, Layout, SettingsContainer } from './Layout';
import List from './List';
import { ModalAlert, ModalAlertType, ModalContainer, ModalMessage } from './Modal';

const StyledCustomScrollbars = styled(CustomScrollbars)({
  flex: 1,
});

const StyledContainer = styled(SettingsContainer)({
  paddingTop: '14px',
  minHeight: '100%',
});

const StyledBody = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 1,
  paddingBottom: 'auto',
});

const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
});

const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  margin: `0 ${measurements.viewMargin} 8px`,
  color: colors.white,
});

const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '12px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white,
  margin: `0 ${measurements.viewMargin} 18px`,
});

const StyledSpacer = styled.div({
  flex: '1',
});

const StyledDeviceName = styled(Cell.Label)({
  textTransform: 'capitalize',
});

const StyledRemoveDeviceButton = styled.button({
  cursor: 'default',
  padding: 0,
  marginLeft: 8,
  backgroundColor: 'transparent',
  border: 'none',
});

export default function TooManyDevices() {
  const history = useHistory();
  const { removeDevice, login, cancelLogin } = useAppContext();
  const accountToken = useSelector((state) => state.account.accountToken)!;
  const devices = useSelector((state) => state.account.devices);
  const loginState = useSelector((state) => state.account.status);

  const onRemoveDevice = useCallback(
    async (deviceId: string) => {
      await removeDevice({ accountToken, deviceId });
    },
    [removeDevice, accountToken],
  );

  const continueLogin = useCallback(() => {
    void login(accountToken);
    history.reset(RoutePath.login, { transition: transitions.pop });
  }, [login, accountToken]);
  const cancel = useCallback(() => {
    cancelLogin();
    history.reset(RoutePath.login, { transition: transitions.pop });
  }, [history.reset, cancelLogin]);

  const iconSource = getIconSource(devices);
  const title = getTitle(devices);
  const subtitle = getSubtitle(devices);

  const continueButtonDisabled = devices.length === 5 || loginState.type !== 'too many devices';

  return (
    <ModalContainer>
      <Layout>
        <Header>
          <Brand />
          <HeaderBarSettingsButton />
        </Header>
        <StyledCustomScrollbars fillContainer>
          <StyledContainer>
            <StyledBody>
              <StyledStatusIcon>
                <ImageView key={iconSource} source={iconSource} height={60} width={60} />
              </StyledStatusIcon>
              {devices !== undefined && (
                <>
                  <StyledTitle>{title}</StyledTitle>
                  <StyledLabel>{subtitle}</StyledLabel>
                  <DeviceList devices={devices} onRemoveDevice={onRemoveDevice} />
                </>
              )}
            </StyledBody>

            {devices !== undefined && (
              <Footer>
                <AppButton.ButtonGroup>
                  <AppButton.GreenButton onClick={continueLogin} disabled={continueButtonDisabled}>
                    {
                      // TRANSLATORS: Button for continuing login process.
                      messages.pgettext('device-management', 'Continue with login')
                    }
                  </AppButton.GreenButton>
                  <AppButton.BlueButton onClick={cancel}>
                    {messages.gettext('Back')}
                  </AppButton.BlueButton>
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
  const { fetchDevices } = useAppContext();
  const accountToken = useSelector((state) => state.account.accountToken)!;
  const [confirmationVisible, showConfirmation, hideConfirmation] = useBoolean(false);
  const [deleting, setDeleting, unsetDeleting] = useBoolean(false);
  const [error, setError, resetError] = useBoolean(false);

  const handleError = useCallback(
    async (error: Error) => {
      log.error(`Failede to remove device: ${error.message}`);

      let devices: Array<IDevice> | undefined = undefined;
      try {
        devices = await fetchDevices(accountToken);
      } catch {
        /* no-op */
      }

      if (devices === undefined || devices.find((device) => device.id === props.device.id)) {
        hideConfirmation();
        unsetDeleting();
        setError();
      }
    },
    [fetchDevices, accountToken, hideConfirmation, setError],
  );

  const onRemove = useCallback(async () => {
    setDeleting();
    hideConfirmation();
    try {
      await props.onRemove(props.device.id);
    } catch (e) {
      await handleError(e as Error);
    }
  }, [props.onRemove, props.device.id, hideConfirmation, setDeleting, handleError]);

  const capitalizedDeviceName = capitalizeEveryWord(props.device.name);

  return (
    <>
      <Cell.Container>
        <StyledDeviceName aria-hidden>{props.device.name}</StyledDeviceName>
        {deleting ? (
          <ImageView source="icon-spinner" width={24} />
        ) : (
          <StyledRemoveDeviceButton
            onClick={showConfirmation}
            aria-label={sprintf(
              // TRANSLATORS: Button action description provided to accessibility tools such as screen
              // TRANSLATORS: readers.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(deviceName)s - The device name to remove.
              messages.pgettext('accessibility', 'Remove device named %(deviceName)s'),
              { deviceName: props.device.name },
            )}>
            <ImageView
              source="icon-close"
              tintColor={colors.white40}
              tintHoverColor={colors.white60}
            />
          </StyledRemoveDeviceButton>
        )}
      </Cell.Container>
      <ModalAlert
        isOpen={confirmationVisible}
        type={ModalAlertType.warning}
        iconColor={colors.red}
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
        {props.device.ports && props.device.ports.length > 0 && (
          <ModalMessage>
            {
              // TRANSLATORS: Further information about consequences of logging out device.
              messages.pgettext(
                'device-management',
                'This will delete all forwarded ports. Local settings will be saved.',
              )
            }
          </ModalMessage>
        )}
      </ModalAlert>
      <ModalAlert
        isOpen={error}
        type={ModalAlertType.warning}
        iconColor={colors.red}
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

function getIconSource(devices?: Array<IDevice>): string {
  if (devices) {
    if (devices.length === 5) {
      return 'icon-fail';
    } else {
      return 'icon-success';
    }
  } else {
    return 'icon-spinner';
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
