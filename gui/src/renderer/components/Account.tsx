import * as React from 'react';

import { formatDate, hasExpired } from '../../shared/account-expiry';
import { AccountToken, DeviceState } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import {
  AccountContainer,
  AccountFooter,
  AccountOutOfTime,
  AccountRow,
  AccountRowLabel,
  AccountRows,
  AccountRowValue,
  DeviceRowValue,
  StyledBuyCreditButton,
  StyledContainer,
  StyledRedeemVoucherButton,
  StyledSpinnerContainer,
} from './AccountStyles';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import { AriaDescribed, AriaDescription, AriaDescriptionGroup } from './AriaGroup';
import ImageView from './ImageView';
import { BackAction } from './KeyboardNavigation';
import { Layout } from './Layout';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import { NavigationBar, NavigationItems, TitleBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';

interface IProps {
  deviceName?: string;
  accountToken?: AccountToken;
  accountExpiry?: string;
  expiryLocale: string;
  isOffline: boolean;
  prepareLogout: () => void;
  cancelLogout: () => void;
  onLogout: () => void;
  onClose: () => void;
  onBuyMore: () => Promise<void>;
  updateAccountData: () => void;
  getDeviceState: () => Promise<DeviceState | undefined>;
}

interface IState {
  logoutDialogState: 'hidden' | 'checking-ports' | 'confirm';
}

export default class Account extends React.Component<IProps, IState> {
  public state: IState = { logoutDialogState: 'hidden' };

  public componentDidMount() {
    this.props.updateAccountData();
  }

  public render() {
    return (
      <BackAction action={this.props.onClose}>
        <Layout>
          <StyledContainer>
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
                  <DeviceRowValue>{this.props.deviceName}</DeviceRowValue>
                </AccountRow>

                <AccountRow>
                  <AccountRowLabel>
                    {messages.pgettext('account-view', 'Account number')}
                  </AccountRowLabel>
                  <AccountRowValue
                    as={AccountTokenLabel}
                    accountToken={this.props.accountToken || ''}
                  />
                </AccountRow>

                <AccountRow>
                  <AccountRowLabel>
                    {messages.pgettext('account-view', 'Paid until')}
                  </AccountRowLabel>
                  <FormattedAccountExpiry
                    expiry={this.props.accountExpiry}
                    locale={this.props.expiryLocale}
                  />
                </AccountRow>
              </AccountRows>

              <AccountFooter>
                <AppButton.BlockingButton
                  disabled={this.props.isOffline}
                  onClick={this.props.onBuyMore}>
                  <AriaDescriptionGroup>
                    <AriaDescribed>
                      <StyledBuyCreditButton>
                        <AppButton.Label>{messages.gettext('Buy more credit')}</AppButton.Label>
                        <AriaDescription>
                          <AppButton.Icon
                            source="icon-extLink"
                            height={16}
                            width={16}
                            aria-label={messages.pgettext('accessibility', 'Opens externally')}
                          />
                        </AriaDescription>
                      </StyledBuyCreditButton>
                    </AriaDescribed>
                  </AriaDescriptionGroup>
                </AppButton.BlockingButton>

                <StyledRedeemVoucherButton />

                <AppButton.RedButton onClick={this.onTryLogout}>
                  {messages.pgettext('account-view', 'Log out')}
                </AppButton.RedButton>
              </AccountFooter>
            </AccountContainer>
          </StyledContainer>

          {this.renderLogoutDialog()}
        </Layout>
      </BackAction>
    );
  }

  private renderLogoutDialog() {
    const modalType =
      this.state.logoutDialogState === 'checking-ports' ? undefined : ModalAlertType.warning;

    const message =
      this.state.logoutDialogState === 'checking-ports' ? (
        <StyledSpinnerContainer>
          <ImageView source="icon-spinner" width={60} height={60} />
        </StyledSpinnerContainer>
      ) : (
        <ModalMessage>
          {
            // TRANSLATORS: This is is a further explanation of what happens when logging out.
            messages.pgettext(
              'device-management',
              'The ports forwarded to this device will be deleted if you log out.',
            )
          }
        </ModalMessage>
      );

    const buttons =
      this.state.logoutDialogState === 'checking-ports'
        ? []
        : [
            <AppButton.RedButton key="logout" onClick={this.props.onLogout}>
              {
                // TRANSLATORS: Confirmation button when logging out
                messages.pgettext('device-management', 'Log out anyway')
              }
            </AppButton.RedButton>,
            <AppButton.BlueButton key="back" onClick={this.cancelLogout}>
              {messages.gettext('Back')}
            </AppButton.BlueButton>,
          ];

    return (
      <ModalAlert
        isOpen={this.state.logoutDialogState !== 'hidden'}
        type={modalType}
        buttons={buttons}>
        {message}
      </ModalAlert>
    );
  }

  private onTryLogout = async () => {
    this.setState({ logoutDialogState: 'checking-ports' });
    this.props.prepareLogout();

    const deviceState = await this.props.getDeviceState();
    if (
      deviceState?.type === 'logged in' &&
      deviceState.accountAndDevice.device?.ports !== undefined &&
      deviceState.accountAndDevice.device.ports.length > 0
    ) {
      this.setState({ logoutDialogState: 'confirm' });
    } else {
      this.props.onLogout();
      this.onHideLogoutConfirmationDialog();
    }
  };

  private cancelLogout = () => {
    this.props.cancelLogout();
    this.onHideLogoutConfirmationDialog();
  };

  private onHideLogoutConfirmationDialog = () => {
    this.setState({ logoutDialogState: 'hidden' });
  };
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
