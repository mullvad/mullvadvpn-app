import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';
import { colors } from '../../config.json';
import { IDevice } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import { transitions, useHistory } from '../lib/history';
import { RoutePath } from '../lib/routes';
import { useBoolean } from '../lib/utilityHooks';
import { useSelector } from '../redux/store';
import * as AppButton from './AppButton';
import * as Cell from './cell';
import { bigText } from './common-styles';
import CustomScrollbars from './CustomScrollbars';
import { Brand, HeaderBarSettingsButton } from './HeaderBar';
import ImageView from './ImageView';
import { Header, Layout, SettingsContainer } from './Layout';
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

const StyledFooter = styled.div({
  display: 'flex',
  flexDirection: 'column',
  flex: 0,
  padding: '18px 22px 22px',
});

const StyledStatusIcon = styled.div({
  alignSelf: 'center',
  width: '60px',
  height: '60px',
  marginBottom: '18px',
});

const StyledTitle = styled.span(bigText, {
  lineHeight: '38px',
  margin: '0 22px 8px',
  color: colors.white,
});

const StyledLabel = styled.span({
  fontFamily: 'Open Sans',
  fontSize: '13px',
  fontWeight: 600,
  lineHeight: '20px',
  color: colors.white,
  margin: '0 22px 18px',
});

const StyledDeviceList = styled(Cell.CellButtonGroup)({
  marginBottom: 0,
  flex: '0 0',
});

const StyledSpacer = styled.div({
  flex: '1',
});

const StyledCellContainer = styled(Cell.Container)({
  marginBottom: '1px',
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
  const { listDevices, removeDevice, login, cancelLogin } = useAppContext();
  const accountToken = useSelector((state) => state.account.accountToken)!;
  const [devices, setDevices] = useState<Array<IDevice>>();

  const fetchDevices = useCallback(async () => {
    setDevices(await listDevices(accountToken));
  }, [listDevices, accountToken]);

  const onRemoveDevice = useCallback(
    async (deviceId: string) => {
      await removeDevice({ accountToken, deviceId });
      await fetchDevices();
    },
    [removeDevice, accountToken],
  );

  const continueLogin = useCallback(() => login(accountToken), [login, accountToken]);
  const cancel = useCallback(() => {
    cancelLogin();
    history.reset(RoutePath.login, transitions.pop);
  }, [history.reset, cancelLogin]);

  useEffect(() => void fetchDevices(), []);

  const iconSource = getIconSource(devices);
  const title = getTitle(devices);
  const subtitle = getSubtitle(devices);

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
              <StyledFooter>
                <AppButton.ButtonGroup>
                  <AppButton.GreenButton onClick={continueLogin} disabled={devices.length === 5}>
                    {
                      // TRANSLATORS: Button for continuing login process.
                      messages.pgettext('device-management', 'Continue with login')
                    }
                  </AppButton.GreenButton>
                  <AppButton.BlueButton onClick={cancel}>
                    {messages.gettext('Back')}
                  </AppButton.BlueButton>
                </AppButton.ButtonGroup>
              </StyledFooter>
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
      <StyledDeviceList>
        <List items={props.devices} getKey={getDeviceKey}>
          {(device) => <Device device={device} onRemove={props.onRemoveDevice} />}
        </List>
      </StyledDeviceList>
    </StyledSpacer>
  );
}

const getDeviceKey = (device: IDevice): string => device.id;

interface IDeviceProps {
  device: IDevice;
  onRemove: (deviceId: string) => Promise<void>;
}

function Device(props: IDeviceProps) {
  const [confirmationVisible, showConfirmation, hideConfirmation] = useBoolean(false);
  const [deleting, setDeleting] = useBoolean(false);

  const onRemove = useCallback(async () => {
    await props.onRemove(props.device.id);
    hideConfirmation();
    setDeleting();
  }, [props.onRemove, props.device.id, hideConfirmation, setDeleting]);

  return (
    <>
      <StyledCellContainer>
        <StyledDeviceName aria-hidden>{props.device.name}</StyledDeviceName>
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
      </StyledCellContainer>
      {confirmationVisible && (
        <ModalAlert
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
          {deleting ? (
            <ImageView source="icon-spinner" />
          ) : (
            <>
              <ModalMessage>
                {sprintf(
                  // TRANSLATORS: Text displayed above button which logs out another device.
                  messages.pgettext(
                    'device-management',
                    'Are you sure you want to log out of %(deviceName)s?',
                  ),
                  { deviceName: props.device.name },
                )}
              </ModalMessage>
              <ModalMessage>
                {
                  // TRANSLATORS: Further information about consequences of logging out device.
                  messages.pgettext(
                    'device-management',
                    'This will delete all forwarded ports. Local settings will be saved.',
                  )
                }
              </ModalMessage>
            </>
          )}
        </ModalAlert>
      )}
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
        'You have too many active devices. Please log out of at least one by removing it from the list below. You can find the corresponding nickname under the device’s Account settings.',
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
