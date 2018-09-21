// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Clipboard, Text, View } from 'reactxp';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import NavigationBar, { BackBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './AccountStyles';
import Img from './Img';

import type { AccountToken } from '../lib/daemon-rpc';

type Props = {
  accountToken: AccountToken,
  accountExpiry: ?string,
  expiryLocale: string,
  onLogout: () => void,
  onClose: () => void,
  onBuyMore: () => void,
};

type State = {
  showAccountTokenCopiedMessage: boolean,
};

type FormattedExpiry = {
  message: string,
  style: boolean,
};

export default class Account extends Component<Props, State> {
  state = {
    showAccountTokenCopiedMessage: false,
  };

  _copyTimer: ?TimeoutID;

  componentWillUnmount() {
    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }
  }

  onAccountTokenClick() {
    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }
    this._copyTimer = setTimeout(
      () => this.setState({ showAccountTokenCopiedMessage: false }),
      3000,
    );
    this.setState({ showAccountTokenCopiedMessage: true });
    Clipboard.setText(this.props.accountToken);
  }

  render() {
    const expiry = this._formatAccountExpiry();

    return (
      <Layout>
        <Container>
          <View style={styles.account}>
            <NavigationBar>
              <BackBarItem action={this.props.onClose} title={'Settings'} />
            </NavigationBar>

            <View style={styles.account__container}>
              <SettingsHeader>
                <HeaderTitle>Account</HeaderTitle>
              </SettingsHeader>

              <View style={styles.account__content}>
                <View style={styles.account__main}>
                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Account ID</Text>
                    <Text
                      style={styles.account__row_value}
                      onPress={this.onAccountTokenClick.bind(this)}>
                      {this.state.showAccountTokenCopiedMessage
                        ? 'COPIED TO CLIPBOARD!'
                        : this.props.accountToken}
                    </Text>
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Paid until</Text>
                    <Text style={expiry.style} testName="account__expiry">
                      {expiry.message}
                    </Text>
                  </View>

                  <View style={styles.account__footer}>
                    <AppButton.GreenButton
                      style={styles.account__buy_button}
                      onPress={this.props.onBuyMore}
                      text="Buy more credit"
                      icon="icon-extLink"
                      testName="account__buymore">
                      <AppButton.Label>Buy more credit</AppButton.Label>
                      <Img source="icon-extLink" height={16} width={16} />
                    </AppButton.GreenButton>
                    <AppButton.RedButton onPress={this.props.onLogout} testName="account__logout">
                      {'Log out'}
                    </AppButton.RedButton>
                  </View>
                </View>
              </View>
            </View>
          </View>
        </Container>
      </Layout>
    );
  }

  _formatAccountExpiry(): FormattedExpiry {
    if (!this.props.accountExpiry) {
      return {
        message: 'Currently unavailable',
        style: styles.account__row_value,
      };
    }

    const expiry = moment(this.props.accountExpiry);

    if (expiry.isSameOrBefore(moment())) {
      return {
        message: 'OUT OF TIME',
        style: styles.account__out_of_time,
      };
    }

    const formatOptions = {
      day: 'numeric',
      month: 'long',
      year: 'numeric',
      hour: 'numeric',
      minute: 'numeric',
    };

    return {
      message: expiry.toDate().toLocaleString(this.props.expiryLocale, formatOptions),
      style: styles.account__row_value,
    };
  }
}
