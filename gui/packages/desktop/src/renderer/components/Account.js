// @flow

import moment from 'moment';
import * as React from 'react';
import { Component, Text, View } from 'reactxp';
import { ClipboardLabel } from '@mullvad/components';
import * as AppButton from './AppButton';
import { Layout, Container } from './Layout';
import { NavigationBar, BackBarItem } from './NavigationBar';
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

export default class Account extends Component<Props> {
  render() {
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
                    <ClipboardLabel
                      style={styles.account__row_value}
                      value={this.props.accountToken}
                      message={'COPIED TO CLIPBOARD!'}
                    />
                  </View>

                  <View style={styles.account__row}>
                    <Text style={styles.account__row_label}>Paid until</Text>
                    <FormattedAccountExpiry
                      expiry={this.props.accountExpiry}
                      locale={this.props.accountExpiryLocale}
                      testName="account__expiry"
                    />
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
}

const FormattedAccountExpiry = (props) => {
  if (!props.expiry) {
    return <Text style={styles.account__row_value}>{'Currently unavailable'}</Text>;
  }

  const expiry = moment(props.expiry);

  if (expiry.isSameOrBefore(moment())) {
    return <Text style={styles.account__out_of_time}>{'OUT OF TIME'}</Text>;
  }

  const formatOptions = {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
  };

  return (
    <Text style={styles.account__row_value}>
      {expiry.toDate().toLocaleString(props.locale, formatOptions)}
    </Text>
  );
};
