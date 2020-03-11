import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { sprintf } from 'sprintf-js';
import { links } from '../../config.json';
import AccountExpiry from '../../shared/account-expiry';
import { AccountToken } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import { LoginState } from '../redux/account/reducers';
import AccountTokenLabel from './AccountTokenLabel';
import * as AppButton from './AppButton';
import * as Cell from './Cell';
import styles from './ExpiredAccountErrorViewStyles';
import ImageView from './ImageView';
import { ModalAlert, ModalAlertType } from './Modal';

export enum RecoveryAction {
  openBrowser,
  disconnect,
  disableBlockedWhenDisconnected,
}

interface IExpiredAccountErrorViewProps {
  isBlocked: boolean;
  blockWhenDisconnected: boolean;
  accountToken?: AccountToken;
  accountExpiry?: AccountExpiry;
  loginState: LoginState;
  hideWelcomeView: () => void;
  onExternalLinkWithAuth: (url: string) => Promise<void>;
  onDisconnect: () => Promise<void>;
  setBlockWhenDisconnected: (value: boolean) => void;
}

interface IExpiredAccountErrorViewState {
  showBlockWhenDisconnectedAlert: boolean;
}

export default class ExpiredAccountErrorView extends Component<
  IExpiredAccountErrorViewProps,
  IExpiredAccountErrorViewState
> {
  public state: IExpiredAccountErrorViewState = {
    showBlockWhenDisconnectedAlert: false,
  };

  public componentDidUpdate() {
    if (this.props.accountExpiry && !this.props.accountExpiry.hasExpired()) {
      this.props.hideWelcomeView();
    }
  }

  public render() {
    return (
      <View style={styles.container}>
        <View style={styles.body}>{this.renderContent()}</View>

        <View style={styles.footer}>
          {this.getRecoveryAction() === RecoveryAction.disconnect && (
            <AppButton.BlockingButton onPress={this.props.onDisconnect}>
              <AppButton.RedButton style={styles.button}>
                {messages.pgettext('connect-view', 'Disconnect')}
              </AppButton.RedButton>
            </AppButton.BlockingButton>
          )}

          {this.renderExternalPaymentButton()}
        </View>

        {this.state.showBlockWhenDisconnectedAlert && this.renderBlockWhenDisconnectedAlert()}
      </View>
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
        <View style={styles.message}>
          <Text style={styles.fieldLabel}>
            {messages.pgettext('connect-view', 'Here’s your account number. Save it!')}
          </Text>
          <AccountTokenLabel
            style={styles.accountToken}
            messageStyle={styles.accountTokenCopiedMessage}
            accountToken={this.props.accountToken || ''}
          />
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
      ? messages.pgettext('connect-view', 'Buy credit')
      : messages.pgettext('connect-view', 'Buy more credit');

    return (
      <AppButton.BlockingButton
        disabled={this.getRecoveryAction() === RecoveryAction.disconnect}
        onPress={this.onOpenExternalPayment}>
        <AppButton.GreenButton>
          <AppButton.Label>{buttonText}</AppButton.Label>
          <AppButton.Icon source="icon-extLink" height={16} width={16} />
        </AppButton.GreenButton>
      </AppButton.BlockingButton>
    );
  }

  private isNewAccount() {
    return this.props.loginState.type === 'ok' && this.props.loginState.method === 'new_account';
  }

  private renderBlockWhenDisconnectedAlert() {
    return (
      <ModalAlert
        type={ModalAlertType.Info}
        buttons={[
          <AppButton.BlueButton
            key="cancel"
            onPress={this.onCloseBlockWhenDisconnectedInstructions}>
            {messages.pgettext('connect-view', 'Close')}
          </AppButton.BlueButton>,
        ]}>
        <Text style={styles.fieldLabel}>
          {messages.pgettext(
            'connect-view',
            'You need to disable “Block when disconnected” in order to access the Internet to add time.',
          )}
        </Text>
        <Text style={styles.fieldLabel}>
          {messages.pgettext(
            'connect-view',
            'Remember, turning it off will allow network traffic while the VPN is disconnected until you turn it back on under Advanced settings.',
          )}
        </Text>
        <Cell.Container>
          <Cell.Label>{messages.pgettext('connect-view', 'Block when disconnected')}</Cell.Label>
          <Cell.Switch
            isOn={this.props.blockWhenDisconnected}
            onChange={this.props.setBlockWhenDisconnected}
          />
        </Cell.Container>
      </ModalAlert>
    );
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

  private onCloseBlockWhenDisconnectedInstructions = () => {
    this.setState({ showBlockWhenDisconnectedAlert: false });
  };
}
