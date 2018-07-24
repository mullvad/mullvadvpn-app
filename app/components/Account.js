// @flow
import moment from 'moment';
import * as React from 'react';
import { Component, Text, View, App, Types } from 'reactxp';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import NavigationBar, { BackBarItem } from './NavigationBar';
import styles from './AccountStyles';
import Img from './Img';
import { formatAccount } from '../lib/formatters';

import type { AccountToken } from '../lib/daemon-rpc';

export type AccountProps = {
  accountToken: AccountToken,
  accountExpiry: string,
  updateAccountExpiry: () => Promise<void>,
  onLogout: () => void,
  onClose: () => void,
  onBuyMore: () => void,
};

export type AccountState = {
  isRefreshingExpiry: boolean,
};

export default class Account extends Component<AccountProps, AccountState> {
  state = {
    isRefreshingExpiry: false,
  };

  _activationStateToken: ?Types.SubscriptionToken;

  _isMounted = false;

  componentDidMount() {
    this._isMounted = true;
    this._refreshAccountExpiry();

    this._activationStateToken = App.activationStateChangedEvent.subscribe((activationState) => {
      if (activationState === Types.AppActivationState.Active) {
        this._refreshAccountExpiry();
      }
    });
  }

  componentWillUnmount() {
    this._isMounted = false;

    const activationStateToken = this._activationStateToken;
    if (activationStateToken) {
      activationStateToken.unsubscribe();
      this._activationStateToken = null;
    }
  }

  render() {
    const expiry = moment(this.props.accountExpiry);
    const formattedAccountToken = formatAccount(this.props.accountToken || '');
    const formattedExpiry = expiry.format('hA, D MMMM YYYY').toUpperCase();
    const isOutOfTime = expiry.isSameOrBefore(moment());

    return (
      <Layout>
        <Container>
          <View style={styles.account}>
            <NavigationBar>
              <BackBarItem action={this.props.onClose} title={'Settings'} />
            </NavigationBar>

            <View style={styles.account__container}>
              <View style={styles.account__header}>
                <Text style={styles.account__title}>Account</Text>
              </View>

              <View style={styles.account__content}>
                <View style={styles.account__main}>
                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Account ID</Text>
                    <Text style={styles.account__row_value}>{formattedAccountToken}</Text>
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Paid until</Text>
                    {isOutOfTime ? (
                      <Text style={styles.account__out_of_time} testName="account__out_of_time">
                        OUT OF TIME
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
