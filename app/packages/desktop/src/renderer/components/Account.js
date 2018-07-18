// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import NavigationBar, { BackBarItem } from './NavigationBar';
import SettingsHeader, { HeaderTitle } from './SettingsHeader';
import styles from './AccountStyles';
import Img from './Img';
import WindowStateObserver from '../lib/window-state-observer';

import type { AccountToken } from '../lib/daemon-rpc';

type Props = {
  accountToken: AccountToken,
  accountExpiry: string,
  expiryLocale: string,
  updateAccountExpiry: () => Promise<void>,
  onLogout: () => void,
  onClose: () => void,
  onCopyAccountToken: () => void,
  onBuyMore: () => void,
};

type State = {
  isRefreshingExpiry: boolean,
  showAccountTokenCopiedMessage: boolean,
};

export default class Account extends Component<Props, State> {
  state = {
    isRefreshingExpiry: false,
    showAccountTokenCopiedMessage: false,
  };

  _isMounted = false;
  _copyTimer: ?TimeoutID;
  _windowStateObserver = new WindowStateObserver();

  componentDidMount() {
    this._isMounted = true;
    this._refreshAccountExpiry();

    this._windowStateObserver.onShow = () => {
      this._refreshAccountExpiry();
    };
  }

  componentWillUnmount() {
    this._isMounted = false;

    if (this._copyTimer) {
      clearTimeout(this._copyTimer);
    }

    this._windowStateObserver.dispose();
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
    this.props.onCopyAccountToken();
  }

  render() {
    const expiry = moment(this.props.accountExpiry);
    const isOutOfTime = expiry.isSameOrBefore(moment());
    const formattedAccountToken = formatAccount(this.props.accountToken || '');
    const formattedExpiry = expiry.toDate().toLocaleString(this.props.expiryLocale, {
      day: 'numeric',
      month: 'long',
      year: 'numeric',
      hour: 'numeric',
      minute: 'numeric',
    });

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
                        : formattedAccountToken}
                    </Text>
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Paid until</Text>
                    {isOutOfTime ? (
                      <Text style={styles.account__out_of_time} testName="account__out_of_time">
                        {'OUT OF TIME'}
                      </Text>
                    ) : (
                      <Text style={styles.account__row_value}>{formattedExpiry}</Text>
                    )}
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

  async _refreshAccountExpiry() {
    this.setState({ isRefreshingExpiry: true });

    try {
      await this.props.updateAccountExpiry();
    } catch (e) {
      // TODO: Report the error to user
    }

    if (this._isMounted) {
      this.setState({ isRefreshingExpiry: false });
    }
  }
}

// TODO: DRY
function formatAccount(val: string) {
  // display number altogether when longer than 12
  if (val.length > 12) {
    return val;
  } else {
    // display quartets
    return val.replace(/([0-9]{4})/g, '$1 ').trim();
  }
}
