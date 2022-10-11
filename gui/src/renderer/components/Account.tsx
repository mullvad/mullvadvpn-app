import { useCallback, useEffect, useState } from 'react';

import { links } from '../../config.json';
import { formatDate, hasExpired } from '../../shared/account-expiry';
import { messages } from '../../shared/gettext';
import { useAppContext } from '../context';
import useActions from '../lib/actionsHook';
import { useHistory } from '../lib/history';
import account from '../redux/account/actions';
import { useSelector } from '../redux/store';
import {
  AccountContainer,
  AccountOutOfTime,
  AccountRow,
  AccountRowLabel,
  AccountRows,
  AccountRowValue,
  DeviceRowValue,
  StyledSpinnerContainer,
} from './AccountStyles';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Footer, Layout, SettingsContainer } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import { NavigationBar, NavigationItems, TitleBarItem } from './NavigationBar';
import { RedeemVoucherButton } from './RedeemVoucher';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

type LogoutDialogStage = 'checking-ports' | 'confirm' | undefined;

export default function Account() {
  const history = useHistory();
  const [logoutDialogStage, setLogoutDialogStage] = useState<LogoutDialogStage>();
  const [logoutDialogVisible, setLogoutDialogVisible] = useState(false);
  const isOffline = useSelector((state) => state.connection.isBlocked);

  const { logout, updateAccountData, openLinkWithAuth, getDeviceState } = useAppContext();

  const { cancelLogout, prepareLogout } = useActions(account);

  const confirmLogout = useCallback(async () => {
    onHideLogoutConfirmationDialog();
    await logout();
  }, []);

  const onCancelLogout = useCallback(() => {
    cancelLogout();
    onHideLogoutConfirmationDialog();
  }, []);

  const onBuyMore = useCallback(async () => {
    await openLinkWithAuth(links.purchase);
  }, []);

  const onHideLogoutConfirmationDialog = () => setLogoutDialogVisible(false);

  const onTryLogout = useCallback(async () => {
    setLogoutDialogVisible(true);
    setLogoutDialogStage('checking-ports');
    prepareLogout();

    const deviceState = await getDeviceState();

    if (
      deviceState?.type === 'logged in' &&
      deviceState.accountAndDevice.device?.ports !== undefined &&
      deviceState.accountAndDevice.device.ports.length > 0
    ) {
      setLogoutDialogStage('confirm');
    } else {
      await confirmLogout();
    }
  }, []);

  useEffect(() => {
    updateAccountData();
  }, []);

  return (
    <BackAction action={history.pop}>
      <Layout>
        <SettingsContainer>
          <NavigationBar>
            <NavigationItems>
              <TitleBarItem>
                {
                  // TRANSLATORS: Title label in navigation bar
                  messages.pgettext('account-view', 'Account')
                }
              </TitleBarItem>
            </NavigationItems>
          </NavigationBar>

          <AccountContainer>
            <SettingsHeader>
              <HeaderTitle>{messages.pgettext('account-view', 'Account')}</HeaderTitle>
            </SettingsHeader>

            <AccountRows>
              <AccountRow>
                <AccountRowLabel>
                  {messages.pgettext('device-management', 'Device name')}
                </AccountRowLabel>
                <DeviceNameRow />
              </AccountRow>

              <AccountRow>
                <AccountRowLabel>
                  {messages.pgettext('account-view', 'Account number')}
                </AccountRowLabel>
                <AccountNumberRow />
              </AccountRow>

              <AccountRow>
                <AccountRowLabel>{messages.pgettext('account-view', 'Paid until')}</AccountRowLabel>
                <AccountExpiryRow />
              </AccountRow>
            </AccountRows>

            <Footer>
              <AppButton.ButtonGroup>
                <AppButton.BlockingButton disabled={isOffline} onClick={onBuyMore}>
                  <AriaDescriptionGroup>
                    <AriaDescribed>
                      <AppButton.GreenButton>
                        <AppButton.Label>{messages.gettext('Buy more credit')}</AppButton.Label>
                        <AriaDescription>
                          <AppButton.Icon
                            source="icon-extLink"
                            height={16}
                            width={16}
                            aria-label={messages.pgettext('accessibility', 'Opens externally')}
                          />
                        </AriaDescription>
                      </AppButton.GreenButton>
                    </AriaDescribed>
                  </AriaDescriptionGroup>
                </AppButton.BlockingButton>

                <RedeemVoucherButton />

                <AppButton.RedButton onClick={onTryLogout}>
                  {messages.pgettext('account-view', 'Log out')}
                </AppButton.RedButton>
              </AppButton.ButtonGroup>
            </Footer>
          </AccountContainer>
        </SettingsContainer>

        <LogoutDialog
          logoutDialogStage={logoutDialogStage}
          logoutDialogVisible={logoutDialogVisible}
          confirmLogout={confirmLogout}
          cancelLogout={onCancelLogout}
        />
      </Layout>
    </BackAction>
  );
}

type LogoutDialogProps = {
  logoutDialogStage: LogoutDialogStage;
  logoutDialogVisible: boolean;
  confirmLogout: () => void;
  cancelLogout: () => void;
};

function LogoutDialog({
  logoutDialogStage,
  logoutDialogVisible,
  confirmLogout,
  cancelLogout,
}: LogoutDialogProps) {
  const modalType = logoutDialogStage === 'checking-ports' ? undefined : ModalAlertType.warning;
  const message =
    logoutDialogStage === 'checking-ports' ? (
      <StyledSpinnerContainer>
        <ImageView source="icon-spinner" width={60} height={60} />
      </StyledSpinnerContainer>
    ) : (
      <ModalMessage>
        {
          // TRANSLATORS: This is a further explanation of what happens when logging out.
          messages.pgettext(
            'device-management',
            'The ports forwarded to this device will be deleted if you log out.',
          )
        }
      </ModalMessage>
    );

  const buttons =
    logoutDialogStage === 'checking-ports'
      ? []
      : [
          <AppButton.RedButton key="logout" onClick={confirmLogout}>
            {
              // TRANSLATORS: Confirmation button when logging out
              messages.pgettext('device-management', 'Log out anyway')
            }
          </AppButton.RedButton>,
          <AppButton.BlueButton key="back" onClick={cancelLogout}>
            {messages.gettext('Back')}
          </AppButton.BlueButton>,
        ];
  return (
    <ModalAlert isOpen={logoutDialogVisible} type={modalType} buttons={buttons}>
      {message}
    </ModalAlert>
  );
}

function DeviceNameRow() {
  const deviceName = useSelector((state) => state.account.deviceName);
  return <DeviceRowValue>{deviceName}</DeviceRowValue>;
}

function AccountNumberRow() {
  const accountToken = useSelector((state) => state.account.accountToken);
  return <AccountRowValue as={AccountTokenLabel} accountToken={accountToken || ''} />;
}

function AccountExpiryRow() {
  const accountExpiry = useSelector((state) => state.account.expiry);
  const expiryLocale = useSelector((state) => state.userInterface.locale);
  return <FormattedAccountExpiry expiry={accountExpiry} locale={expiryLocale} />;
}

function FormattedAccountExpiry(props: { expiry?: string; locale: string }) {
  if (props.expiry) {
    if (hasExpired(props.expiry)) {
      return (
        <AccountOutOfTime>{messages.pgettext('account-view', 'OUT OF TIME')}</AccountOutOfTime>
      );
    } else {
      return <AccountRowValue>{formatDate(props.expiry, props.locale)}</AccountRowValue>;
    }
  } else {
    return (
      <AccountRowValue>
        {messages.pgettext('account-view', 'Currently unavailable')}
      </AccountRowValue>
    );
  }
}
