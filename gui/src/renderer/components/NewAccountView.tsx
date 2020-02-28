import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { links } from '../../config.json';
import AccountExpiry from '../../shared/account-expiry';
import { AccountToken } from '../../shared/daemon-rpc-types';
import { messages } from '../../shared/gettext';
import RedeemVoucherContainer from '../containers/RedeemVoucherContainer';
import styles from './NewAccountViewStyles';
import { AccountTokenLabel } from './Account';
import * as AppButton from './AppButton';
import { ModalAlert } from './Modal';
import {
  RedeemVoucherInput,
  RedeemVoucherResponse,
  RedeemVoucherSubmitButton,
} from './RedeemVoucher';

interface INewAccountViewProps {
  accountToken?: AccountToken;
  accountExpiry?: AccountExpiry;
  updateAccountData: () => void;
  hideWelcomeView: () => void;
  onExternalLinkWithAuth: (url: string) => Promise<void>;
}

interface INewAccountViewState {
  showRedeemVoucherAlert: boolean;
  redeemingVoucher: boolean;
}

export default class NewAccountView extends Component<INewAccountViewProps, INewAccountViewState> {
  state = {
    showRedeemVoucherAlert: false,
    redeemingVoucher: false,
  };

  private updateAccountDataInterval?: number;

  public componentDidMount() {
    this.updateAccountDataInterval = setInterval(this.props.updateAccountData, 30 * 1000);
  }

  public componentDidUpdate() {
    if (this.props.accountExpiry && !this.props.accountExpiry.hasExpired()) {
      this.props.hideWelcomeView();
    }
  }

  public componentWillUnmount() {
    clearInterval(this.updateAccountDataInterval);
  }

  public render() {
    return (
      <View style={styles.container}>
        <View style={styles.body}>
          <View style={styles.title}>{messages.pgettext('new-account-view', 'Congrats!')}</View>
          <View style={styles.message}>
            <Text style={styles.fieldLabel}>
              {messages.pgettext('new-account-view', "Here's your account number! Save it!")}
            </Text>
            <AccountTokenLabel
              style={styles.accountToken}
              accountToken={this.props.accountToken || ''}
            />
          </View>

          <View style={styles.message}>
            {messages.pgettext(
              'new-account-view',
              'To start using the app you first need to add time to you account. You can either buy it online or redeem a voucher if you have one.',
            )}
          </View>
        </View>
        {this.createFooter()}

        {this.state.showRedeemVoucherAlert && this.renderRedeemVoucherAlert()}
      </View>
    );
  }

  private createFooter() {
    return (
      <View style={styles.footer}>
        <AppButton.BlockingButton onPress={this.openPaymentUrl}>
          <AppButton.GreenButton style={styles.buyOnlineButton}>
            <AppButton.Label>{messages.pgettext('new-account-view', 'Buy online')}</AppButton.Label>
            <AppButton.Icon source="icon-extLink" height={16} width={16} />
          </AppButton.GreenButton>
        </AppButton.BlockingButton>

        <AppButton.GreenButton onPress={this.onOpenRedeemVoucherAlert}>
          {messages.pgettext('new-account-view', 'Redeem voucher')}
        </AppButton.GreenButton>
      </View>
    );
  }

  private renderRedeemVoucherAlert() {
    return (
      <RedeemVoucherContainer
        onSubmit={this.onVoucherSubmit}
        onSuccess={this.props.hideWelcomeView}
        onFailure={this.onVoucherResponse}>
        <ModalAlert
          buttons={[
            <RedeemVoucherSubmitButton key="submit" />,
            <AppButton.BlueButton
              key="cancel"
              disabled={this.state.redeemingVoucher}
              onPress={this.onCloseRedeemVoucherAlert}>
              {messages.pgettext('new-account-view', 'Cancel')}
            </AppButton.BlueButton>,
          ]}>
          <Text style={styles.fieldLabel}>
            {messages.pgettext('new-account-view', 'Enter voucher code')}
          </Text>
          <RedeemVoucherInput />
          <RedeemVoucherResponse />
        </ModalAlert>
      </RedeemVoucherContainer>
    );
  }

  private onOpenRedeemVoucherAlert = () => {
    this.setState({ showRedeemVoucherAlert: true });
  };

  private onCloseRedeemVoucherAlert = () => {
    this.setState({ showRedeemVoucherAlert: false });
  };

  private onVoucherSubmit = () => {
    this.setState({ redeemingVoucher: true });
  };

  private onVoucherResponse = () => {
    this.setState({ redeemingVoucher: false });
  };

  private openPaymentUrl = async (): Promise<void> => {
    await this.props.onExternalLinkWithAuth(links.purchase);
  };
}
