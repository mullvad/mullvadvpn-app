import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import { hasExpired } from '../../shared/account-expiry';
import { AccountToken } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { LoginState } from '../redux/account/reducers';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import CustomScrollbars from './CustomScrollbars';
import styles, {
  ModalCellContainer,
  StyledAccountTokenLabel,
  StyledBuyCreditButton,
  StyledDisconnectButton,
} from './ExpiredAccountErrorViewStyles';
import ImageView from './ImageView';
import { ModalAlert, ModalAlertType, ModalMessage } from './Modal';
import { RedeemVoucherContainer, RedeemVoucherAlert } from './RedeemVoucher';

export enum RecoveryAction {
  openBrowser,
  disconnect,
  disableBlockedWhenDisconnected,
}

interface IExpiredAccountErrorViewProps {
  isBlocked: boolean;
  blockWhenDisconnected: boolean;
  accountToken?: AccountToken;
  accountExpiry?: string;
  loginState: LoginState;
  hideWelcomeView: () => void;
  onExternalLinkWithAuth: (url: string) => Promise<void>;
  onDisconnect: () => Promise<void>;
  setBlockWhenDisconnected: (value: boolean) => void;
}

interface IExpiredAccountErrorViewState {
  showBlockWhenDisconnectedAlert: boolean;
  showRedeemVoucherAlert: boolean;
}

export default class ExpiredAccountErrorView extends Component<
  IExpiredAccountErrorViewProps,
  IExpiredAccountErrorViewState
> {
  public state: IExpiredAccountErrorViewState = {
    showBlockWhenDisconnectedAlert: false,
    showRedeemVoucherAlert: false,
  };

  public componentDidUpdate() {
    if (this.props.accountExpiry && !hasExpired(this.props.accountExpiry)) {
      this.props.hideWelcomeView();
    }
  }

  public render() {
    return (
      <CustomScrollbars style={styles.scrollview} fillContainer>
        <View style={styles.container}>
          <View style={styles.body}>{this.renderContent()}</View>

          <View style={styles.footer}>
            {this.getRecoveryAction() === RecoveryAction.disconnect && (
              <AppButton.BlockingButton onClick={this.props.onDisconnect}>
                <StyledDisconnectButton>
                  {messages.pgettext('connect-view', 'Disconnect')}
                </StyledDisconnectButton>
              </AppButton.BlockingButton>
            )}

            {this.renderExternalPaymentButton()}

            <AppButton.GreenButton
              disabled={this.getRecoveryAction() === RecoveryAction.disconnect}
              onClick={this.onOpenRedeemVoucherAlert}>
              {messages.pgettext('connect-view', 'Redeem voucher')}
            </AppButton.GreenButton>
          </View>

          {this.state.showRedeemVoucherAlert && this.renderRedeemVoucherAlert()}
          {this.state.showBlockWhenDisconnectedAlert && this.renderBlockWhenDisconnectedAlert()}
        </View>
      </CustomScrollbars>
    );
  }

  private renderContent() {
    if (this.isNewAccount()) {
      return this.renderWelcomeView();
    }

    return (
      <>
        <View style={styles.statusIcon}>
          <ImageView source="icon-fail" height={60} width={60} />
        </View>
        <View style={styles.title}>{messages.pgettext('connect-view', 'Out of time')}</View>
        <View style={styles.message}>
          {sprintf('%(introduction)s %(recoveryMessage)s', {
            introduction: messages.pgettext(
              'connect-view',
              'You have no more VPN time left on this account.',
            ),
            recoveryMessage: this.getRecoveryActionMessage(),
          })}
        </View>
      </>
    );
  }

  private renderWelcomeView() {
    return (
      <>
        <View style={styles.title}>{messages.pgettext('connect-view', 'Congrats!')}</View>
        <View style={[styles.message, styles.accountTokenMessage]}>
          <Text style={[styles.fieldLabel, styles.accountTokenFieldLabel]}>
            {messages.pgettext('connect-view', 'Here’s your account number. Save it!')}
          </Text>
          <View style={styles.accountTokenContainer}>
            <StyledAccountTokenLabel accountToken={this.props.accountToken || ''} />
          </View>
        </View>

        <View style={styles.message}>
          {sprintf('%(introduction)s %(recoveryMessage)s', {
            introduction: messages.pgettext(
              'connect-view',
              'To start using the app, you first need to add time to your account.',
            ),
            recoveryMessage: this.getRecoveryActionMessage(),
          })}
        </View>
      </>
    );
  }

  private getRecoveryActionMessage() {
    switch (this.getRecoveryAction()) {
      case RecoveryAction.openBrowser:
      case RecoveryAction.disableBlockedWhenDisconnected:
        return messages.pgettext(
          'connect-view',
          'Either buy credit on our website or redeem a voucher.',
        );
      case RecoveryAction.disconnect:
        return messages.pgettext(
          'connect-view',
          'To add more, you will need to disconnect and access the Internet with an unsecured connection.',
        );
    }
  }

  private renderExternalPaymentButton() {
    const buttonText = this.isNewAccount()
      ? messages.gettext('Buy credit')
      : messages.gettext('Buy more credit');

    return (
      <AppButton.BlockingButton
        disabled={this.getRecoveryAction() === RecoveryAction.disconnect}
        onClick={this.onOpenExternalPayment}>
        <StyledBuyCreditButton>
          <AppButton.Label>{buttonText}</AppButton.Label>
          <AppButton.Icon source="icon-extLink" height={16} width={16} />
        </StyledBuyCreditButton>
      </AppButton.BlockingButton>
    );
  }

  private renderRedeemVoucherAlert() {
    return (
      <RedeemVoucherContainer onSuccess={this.props.hideWelcomeView}>
        <RedeemVoucherAlert onClose={this.onCloseRedeemVoucherAlert} />
      </RedeemVoucherContainer>
    );
  }

  private renderBlockWhenDisconnectedAlert() {
    return (
      <ModalAlert
        type={ModalAlertType.Info}
        buttons={[
          <AppButton.BlueButton
            key="cancel"
            onClick={this.onCloseBlockWhenDisconnectedInstructions}>
            {messages.gettext('Close')}
          </AppButton.BlueButton>,
        ]}>
        <ModalMessage>
          {messages.pgettext(
            'connect-view',
            'You need to disable "Always require VPN" in order to access the Internet to add time.',
          )}
        </ModalMessage>
        <ModalMessage>
          {messages.pgettext(
            'connect-view',
            'Remember, turning it off will allow network traffic while the VPN is disconnected until you turn it back on under Advanced settings.',
          )}
        </ModalMessage>
        <ModalCellContainer>
          <Cell.Label>{messages.pgettext('connect-view', 'Always require VPN')}</Cell.Label>
          <Cell.Switch
            isOn={this.props.blockWhenDisconnected}
            onChange={this.props.setBlockWhenDisconnected}
          />
        </ModalCellContainer>
      </ModalAlert>
    );
  }

  private isNewAccount() {
    return this.props.loginState.type === 'ok' && this.props.loginState.method === 'new_account';
  }

  private onOpenExternalPayment = async (): Promise<void> => {
    if (this.getRecoveryAction() === RecoveryAction.disableBlockedWhenDisconnected) {
      this.setState({ showBlockWhenDisconnectedAlert: true });
    } else {
      await this.props.onExternalLinkWithAuth(links.purchase);
    }
  };

  private getRecoveryAction() {
    const { blockWhenDisconnected, isBlocked } = this.props;

    if (blockWhenDisconnected && isBlocked) {
      return RecoveryAction.disableBlockedWhenDisconnected;
    } else if (!blockWhenDisconnected && isBlocked) {
      return RecoveryAction.disconnect;
    } else {
      return RecoveryAction.openBrowser;
    }
  }

  private onOpenRedeemVoucherAlert = () => {
    if (this.getRecoveryAction() === RecoveryAction.disableBlockedWhenDisconnected) {
      this.setState({ showBlockWhenDisconnectedAlert: true });
    } else {
      this.setState({ showRedeemVoucherAlert: true });
    }
  };

  private onCloseRedeemVoucherAlert = () => {
    this.setState({ showRedeemVoucherAlert: false });
  };

  private onCloseBlockWhenDisconnectedInstructions = () => {
    this.setState({ showBlockWhenDisconnectedAlert: false });
  };
}
